use anyhow::{anyhow, bail, Context, Result};
use async_trait::async_trait;
use sdkwork_api_cache_core::{CacheEntry, CacheStore, CacheTag, DistributedLockStore};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;

pub struct RedisCacheStore {
    config: RedisConnectionConfig,
}

impl RedisCacheStore {
    pub async fn connect(cache_url: &str) -> Result<Self> {
        let config = RedisConnectionConfig::parse(cache_url)?;
        let store = Self { config };
        store.ping().await?;
        Ok(store)
    }

    async fn ping(&self) -> Result<()> {
        let mut connection = self.open_connection().await?;
        connection.expect_simple_string(&[b"PING".to_vec()], "PONG").await
    }

    async fn open_connection(&self) -> Result<RedisConnection> {
        let stream = TcpStream::connect((self.config.host.as_str(), self.config.port))
            .await
            .with_context(|| {
                format!(
                    "failed to connect to redis cache backend {}:{}",
                    self.config.host, self.config.port
                )
            })?;
        let mut connection = RedisConnection { stream };

        if let Some(password) = self.config.password.as_deref() {
            let mut command = vec![b"AUTH".to_vec()];
            if let Some(username) = self.config.username.as_deref() {
                command.push(username.as_bytes().to_vec());
            }
            command.push(password.as_bytes().to_vec());
            connection.expect_ok(&command).await?;
        }

        if self.config.db != 0 {
            connection
                .expect_ok(&[b"SELECT".to_vec(), self.config.db.to_string().into_bytes()])
                .await?;
        }

        Ok(connection)
    }

    async fn cleanup_entry_indices(
        &self,
        connection: &mut RedisConnection,
        namespace: &str,
        storage_key: &str,
    ) -> Result<bool> {
        let reverse_tags_key = reverse_tags_key(storage_key);
        let tags = connection
            .smembers(reverse_tags_key.as_bytes())
            .await
            .with_context(|| format!("failed to read redis reverse tag index for {storage_key}"))?;
        for tag in &tags {
            let tag_key = tag_members_key(namespace, &String::from_utf8_lossy(tag));
            connection
                .srem(tag_key.as_bytes(), &[storage_key.as_bytes().to_vec()])
                .await
                .with_context(|| format!("failed to update redis tag index for {storage_key}"))?;
        }

        let removed = connection
            .del(&[storage_key.as_bytes().to_vec()])
            .await
            .with_context(|| format!("failed to delete redis cache key {storage_key}"))?
            > 0;
        let _ = connection.del(&[reverse_tags_key.into_bytes()]).await?;
        Ok(removed)
    }
}

#[async_trait]
impl CacheStore for RedisCacheStore {
    async fn get(&self, namespace: &str, key: &str) -> Result<Option<CacheEntry>> {
        let storage_key = entry_storage_key(namespace, key);
        let mut connection = self.open_connection().await?;
        let value = connection.get(storage_key.as_bytes()).await?;
        if let Some(value) = value {
            return Ok(Some(CacheEntry::new(value)));
        }

        let _ = self
            .cleanup_entry_indices(&mut connection, namespace, &storage_key)
            .await?;
        Ok(None)
    }

    async fn put(
        &self,
        namespace: &str,
        key: &str,
        value: Vec<u8>,
        ttl_ms: Option<u64>,
        tags: &[CacheTag],
    ) -> Result<()> {
        let storage_key = entry_storage_key(namespace, key);
        let reverse_tags_key = reverse_tags_key(&storage_key);
        let mut connection = self.open_connection().await?;

        let _ = self
            .cleanup_entry_indices(&mut connection, namespace, &storage_key)
            .await?;

        connection
            .set(storage_key.as_bytes(), &value, ttl_ms, false)
            .await
            .with_context(|| format!("failed to write redis cache key {storage_key}"))?;

        if !tags.is_empty() {
            let tag_values = tags
                .iter()
                .map(|tag| tag.value().as_bytes().to_vec())
                .collect::<Vec<_>>();
            connection
                .sadd(reverse_tags_key.as_bytes(), &tag_values)
                .await
                .with_context(|| {
                    format!("failed to persist redis reverse tag index for {storage_key}")
                })?;

            for tag in tags {
                let tag_key = tag_members_key(namespace, tag.value());
                connection
                    .sadd(tag_key.as_bytes(), &[storage_key.as_bytes().to_vec()])
                    .await
                    .with_context(|| format!("failed to persist redis tag index {tag_key}"))?;
            }
        }

        Ok(())
    }

    async fn delete(&self, namespace: &str, key: &str) -> Result<bool> {
        let storage_key = entry_storage_key(namespace, key);
        let mut connection = self.open_connection().await?;
        self.cleanup_entry_indices(&mut connection, namespace, &storage_key)
            .await
    }

    async fn invalidate_tag(&self, namespace: &str, tag: &str) -> Result<u64> {
        let tag_key = tag_members_key(namespace, tag);
        let mut connection = self.open_connection().await?;
        let members = connection.smembers(tag_key.as_bytes()).await?;
        let mut removed = 0_u64;
        for member in members {
            let storage_key = String::from_utf8(member).context("redis tag member is not utf8")?;
            if self
                .cleanup_entry_indices(&mut connection, namespace, &storage_key)
                .await?
            {
                removed += 1;
            }
        }
        let _ = connection.del(&[tag_key.into_bytes()]).await?;
        Ok(removed)
    }
}

#[async_trait]
impl DistributedLockStore for RedisCacheStore {
    async fn try_acquire_lock(&self, scope: &str, owner: &str, ttl_ms: u64) -> Result<bool> {
        let lock_key = lock_key(scope);
        let mut connection = self.open_connection().await?;
        connection
            .set(lock_key.as_bytes(), owner.as_bytes(), Some(ttl_ms), true)
            .await
    }

    async fn release_lock(&self, scope: &str, owner: &str) -> Result<bool> {
        let lock_key = lock_key(scope);
        let mut connection = self.open_connection().await?;
        let current = connection.get(lock_key.as_bytes()).await?;
        match current {
            Some(current) if current == owner.as_bytes() => {
                Ok(connection.del(&[lock_key.into_bytes()]).await? > 0)
            }
            _ => Ok(false),
        }
    }
}

#[derive(Debug, Clone)]
struct RedisConnectionConfig {
    username: Option<String>,
    password: Option<String>,
    host: String,
    port: u16,
    db: u32,
}

impl RedisConnectionConfig {
    fn parse(cache_url: &str) -> Result<Self> {
        let rest = cache_url
            .strip_prefix("redis://")
            .ok_or_else(|| anyhow!("redis cache url must start with redis://"))?;
        let (authority, path_and_query) = match rest.split_once('/') {
            Some((authority, path_and_query)) => (authority, path_and_query),
            None => (rest, ""),
        };

        let (username, password, host_port) = match authority.rsplit_once('@') {
            Some((auth, host_port)) => {
                let (username, password) = match auth.split_once(':') {
                    Some((username, password)) if !username.is_empty() => {
                        (Some(username.to_owned()), Some(password.to_owned()))
                    }
                    Some((_, password)) => (None, Some(password.to_owned())),
                    None if !auth.is_empty() => (None, Some(auth.to_owned())),
                    None => (None, None),
                };
                (username, password, host_port)
            }
            None => (None, None, authority),
        };

        let (host, port) = parse_host_port(host_port)?;
        let db = parse_database_index(path_and_query)?;

        Ok(Self {
            username,
            password,
            host,
            port,
            db,
        })
    }
}

fn parse_host_port(host_port: &str) -> Result<(String, u16)> {
    if let Some(stripped) = host_port.strip_prefix('[') {
        let (host, remainder) = stripped
            .split_once(']')
            .ok_or_else(|| anyhow!("redis cache url has invalid ipv6 host"))?;
        let port = remainder
            .strip_prefix(':')
            .map(str::parse::<u16>)
            .transpose()?
            .unwrap_or(6379);
        return Ok((host.to_owned(), port));
    }

    match host_port.rsplit_once(':') {
        Some((host, port)) if !host.is_empty() && !port.is_empty() => {
            Ok((host.to_owned(), port.parse::<u16>()?))
        }
        _ => Ok((host_port.to_owned(), 6379)),
    }
}

fn parse_database_index(path_and_query: &str) -> Result<u32> {
    let database = path_and_query
        .split('?')
        .next()
        .unwrap_or_default()
        .trim_matches('/');
    if database.is_empty() {
        return Ok(0);
    }
    Ok(database.parse::<u32>()?)
}

struct RedisConnection {
    stream: TcpStream,
}

impl RedisConnection {
    async fn expect_ok(&mut self, command: &[Vec<u8>]) -> Result<()> {
        match self.send_command(command).await? {
            RespValue::SimpleString(value) if value == "OK" => Ok(()),
            other => bail!("unexpected redis response: {}", other.kind_name()),
        }
    }

    async fn expect_simple_string(&mut self, command: &[Vec<u8>], expected: &str) -> Result<()> {
        match self.send_command(command).await? {
            RespValue::SimpleString(value) if value == expected => Ok(()),
            other => bail!("unexpected redis response: {}", other.kind_name()),
        }
    }

    async fn get(&mut self, key: &[u8]) -> Result<Option<Vec<u8>>> {
        match self.send_command(&[b"GET".to_vec(), key.to_vec()]).await? {
            RespValue::BulkString(value) => Ok(value),
            other => bail!("unexpected redis GET response: {}", other.kind_name()),
        }
    }

    async fn set(
        &mut self,
        key: &[u8],
        value: &[u8],
        ttl_ms: Option<u64>,
        nx: bool,
    ) -> Result<bool> {
        let mut command = vec![b"SET".to_vec(), key.to_vec(), value.to_vec()];
        if nx {
            command.push(b"NX".to_vec());
        }
        if let Some(ttl_ms) = ttl_ms {
            command.push(b"PX".to_vec());
            command.push(ttl_ms.to_string().into_bytes());
        }

        match self.send_command(&command).await? {
            RespValue::SimpleString(value) if value == "OK" => Ok(true),
            RespValue::BulkString(None) => Ok(false),
            other => bail!("unexpected redis SET response: {}", other.kind_name()),
        }
    }

    async fn del(&mut self, keys: &[Vec<u8>]) -> Result<i64> {
        let mut command = Vec::with_capacity(keys.len() + 1);
        command.push(b"DEL".to_vec());
        command.extend(keys.iter().cloned());
        match self.send_command(&command).await? {
            RespValue::Integer(value) => Ok(value),
            other => bail!("unexpected redis DEL response: {}", other.kind_name()),
        }
    }

    async fn sadd(&mut self, key: &[u8], members: &[Vec<u8>]) -> Result<i64> {
        let mut command = Vec::with_capacity(members.len() + 2);
        command.push(b"SADD".to_vec());
        command.push(key.to_vec());
        command.extend(members.iter().cloned());
        match self.send_command(&command).await? {
            RespValue::Integer(value) => Ok(value),
            other => bail!("unexpected redis SADD response: {}", other.kind_name()),
        }
    }

    async fn smembers(&mut self, key: &[u8]) -> Result<Vec<Vec<u8>>> {
        match self.send_command(&[b"SMEMBERS".to_vec(), key.to_vec()]).await? {
            RespValue::Array(values) => Ok(values),
            other => bail!("unexpected redis SMEMBERS response: {}", other.kind_name()),
        }
    }

    async fn srem(&mut self, key: &[u8], members: &[Vec<u8>]) -> Result<i64> {
        let mut command = Vec::with_capacity(members.len() + 2);
        command.push(b"SREM".to_vec());
        command.push(key.to_vec());
        command.extend(members.iter().cloned());
        match self.send_command(&command).await? {
            RespValue::Integer(value) => Ok(value),
            other => bail!("unexpected redis SREM response: {}", other.kind_name()),
        }
    }

    async fn send_command(&mut self, command: &[Vec<u8>]) -> Result<RespValue> {
        let payload = encode_command(command);
        self.stream.write_all(&payload).await?;
        self.stream.flush().await?;
        let response = read_resp_value(&mut self.stream).await?;
        if let RespValue::Error(message) = &response {
            bail!("redis command failed: {message}");
        }
        Ok(response)
    }
}

fn encode_command(command: &[Vec<u8>]) -> Vec<u8> {
    let mut payload = Vec::new();
    payload.extend_from_slice(format!("*{}\r\n", command.len()).as_bytes());
    for part in command {
        payload.extend_from_slice(format!("${}\r\n", part.len()).as_bytes());
        payload.extend_from_slice(part);
        payload.extend_from_slice(b"\r\n");
    }
    payload
}

async fn read_resp_value(stream: &mut TcpStream) -> Result<RespValue> {
    let mut marker = [0_u8; 1];
    stream.read_exact(&mut marker).await?;
    match marker[0] {
        b'+' => Ok(RespValue::SimpleString(read_resp_line(stream).await?)),
        b'-' => Ok(RespValue::Error(read_resp_line(stream).await?)),
        b':' => Ok(RespValue::Integer(read_resp_line(stream).await?.parse()?)),
        b'$' => {
            let length = read_resp_line(stream).await?.parse::<i64>()?;
            if length < 0 {
                return Ok(RespValue::BulkString(None));
            }
            let mut value = vec![0_u8; length as usize];
            stream.read_exact(&mut value).await?;
            let mut crlf = [0_u8; 2];
            stream.read_exact(&mut crlf).await?;
            Ok(RespValue::BulkString(Some(value)))
        }
        b'*' => {
            let count = read_resp_line(stream).await?.parse::<i64>()?;
            if count < 0 {
                return Ok(RespValue::Array(Vec::new()));
            }
            let mut values = Vec::with_capacity(count as usize);
            for _ in 0..count {
                let mut bulk_marker = [0_u8; 1];
                stream.read_exact(&mut bulk_marker).await?;
                if bulk_marker[0] != b'$' {
                    bail!("unsupported nested redis array item marker: {}", bulk_marker[0] as char);
                }
                let length = read_resp_line(stream).await?.parse::<i64>()?;
                if length < 0 {
                    values.push(Vec::new());
                    continue;
                }
                let mut value = vec![0_u8; length as usize];
                stream.read_exact(&mut value).await?;
                let mut crlf = [0_u8; 2];
                stream.read_exact(&mut crlf).await?;
                values.push(value);
            }
            Ok(RespValue::Array(values))
        }
        other => bail!("unsupported redis response marker: {}", other as char),
    }
}

async fn read_resp_line(stream: &mut TcpStream) -> Result<String> {
    let mut bytes = Vec::new();
    loop {
        let mut byte = [0_u8; 1];
        stream.read_exact(&mut byte).await?;
        if byte[0] == b'\r' {
            let mut newline = [0_u8; 1];
            stream.read_exact(&mut newline).await?;
            if newline[0] != b'\n' {
                bail!("invalid redis response line ending");
            }
            break;
        }
        bytes.push(byte[0]);
    }
    String::from_utf8(bytes).context("redis response line is not utf8")
}

enum RespValue {
    SimpleString(String),
    Error(String),
    Integer(i64),
    BulkString(Option<Vec<u8>>),
    Array(Vec<Vec<u8>>),
}

impl RespValue {
    fn kind_name(&self) -> &'static str {
        match self {
            Self::SimpleString(_) => "simple-string",
            Self::Error(_) => "error",
            Self::Integer(_) => "integer",
            Self::BulkString(_) => "bulk-string",
            Self::Array(_) => "array",
        }
    }
}

fn entry_storage_key(namespace: &str, key: &str) -> String {
    format!("sdkwork:cache:entry:{namespace}:{key}")
}

fn reverse_tags_key(storage_key: &str) -> String {
    format!("sdkwork:cache:entry-tags:{storage_key}")
}

fn tag_members_key(namespace: &str, tag: &str) -> String {
    format!("sdkwork:cache:tag:{namespace}:{tag}")
}

fn lock_key(scope: &str) -> String {
    format!("sdkwork:cache:lock:{scope}")
}
