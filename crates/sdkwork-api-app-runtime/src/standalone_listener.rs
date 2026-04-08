use super::*;

pub struct StandaloneListenerHost {
    inner: Arc<StandaloneListenerHostInner>,
    exit_rx: mpsc::UnboundedReceiver<ListenerServerExit>,
}

#[derive(Clone)]
pub struct StandaloneListenerHandle {
    inner: Arc<StandaloneListenerHostInner>,
}

struct StandaloneListenerHostInner {
    router: Router,
    active: Mutex<Option<RunningStandaloneListenerServer>>,
    exit_tx: mpsc::UnboundedSender<ListenerServerExit>,
    next_generation: AtomicU64,
}

struct RunningStandaloneListenerServer {
    bind: String,
    generation: u64,
    shutdown_requested: Arc<AtomicBool>,
    shutdown_tx: Option<oneshot::Sender<()>>,
    join_handle: JoinHandle<()>,
}

struct ListenerServerExit {
    generation: u64,
    bind: String,
    shutdown_requested: bool,
    result: io::Result<()>,
}

pub(crate) struct PreparedStandaloneListenerRebind {
    inner: Arc<StandaloneListenerHostInner>,
    bind: String,
    listener: TcpListener,
}

impl StandaloneListenerHost {
    pub async fn bind(bind: impl Into<String>, router: Router) -> Result<Self> {
        let bind = bind.into();
        let listener = TcpListener::bind(&bind)
            .await
            .with_context(|| format!("failed to bind standalone listener to {bind}"))?;
        let actual_bind = listener
            .local_addr()
            .with_context(|| format!("failed to resolve standalone listener bind for {bind}"))?
            .to_string();
        let (exit_tx, exit_rx) = mpsc::unbounded_channel();
        let inner = Arc::new(StandaloneListenerHostInner {
            router,
            active: Mutex::new(None),
            exit_tx,
            next_generation: AtomicU64::new(1),
        });
        inner.activate_prebound(actual_bind, listener);

        Ok(Self { inner, exit_rx })
    }

    pub fn reload_handle(&self) -> StandaloneListenerHandle {
        StandaloneListenerHandle {
            inner: self.inner.clone(),
        }
    }

    pub fn current_bind(&self) -> Option<String> {
        self.inner.current_bind()
    }

    pub async fn shutdown(mut self) -> Result<()> {
        let Some(active) = self.inner.take_active_server() else {
            return Ok(());
        };
        let generation = active.generation;
        let bind = active.bind.clone();
        active.request_shutdown();

        while let Some(exit) = self.exit_rx.recv().await {
            if exit.generation != generation {
                continue;
            }

            return exit
                .result
                .with_context(|| format!("listener shutdown failed for bind {bind}"));
        }

        anyhow::bail!("listener host closed before shutdown completed for bind {bind}");
    }

    pub async fn wait(mut self) -> Result<()> {
        while let Some(exit) = self.exit_rx.recv().await {
            if exit.shutdown_requested {
                if let Err(error) = exit.result {
                    eprintln!(
                        "standalone listener shutdown completed with error: bind={} error={error}",
                        exit.bind
                    );
                }
                continue;
            }

            return match exit.result {
                Ok(()) => anyhow::bail!(
                    "standalone listener exited unexpectedly without a shutdown request: bind={}",
                    exit.bind
                ),
                Err(error) => Err(anyhow::Error::new(error).context(format!(
                    "standalone listener exited unexpectedly: bind={}",
                    exit.bind
                ))),
            };
        }

        anyhow::bail!("standalone listener host closed unexpectedly");
    }
}

impl StandaloneListenerHandle {
    pub fn current_bind(&self) -> Option<String> {
        self.inner.current_bind()
    }

    pub async fn rebind(&self, bind: impl Into<String>) -> Result<()> {
        if let Some(prepared) = self.prepare_rebind(bind).await? {
            prepared.activate();
        }
        Ok(())
    }

    pub(crate) async fn prepare_rebind(
        &self,
        bind: impl Into<String>,
    ) -> Result<Option<PreparedStandaloneListenerRebind>> {
        let bind = bind.into();
        if self.current_bind().as_deref() == Some(bind.as_str()) {
            return Ok(None);
        }

        let listener = TcpListener::bind(&bind)
            .await
            .with_context(|| format!("failed to bind replacement standalone listener to {bind}"))?;
        let actual_bind = listener
            .local_addr()
            .with_context(|| {
                format!("failed to resolve replacement standalone listener bind for {bind}")
            })?
            .to_string();

        Ok(Some(PreparedStandaloneListenerRebind {
            inner: self.inner.clone(),
            bind: actual_bind,
            listener,
        }))
    }
}

impl StandaloneListenerHostInner {
    fn activate_prebound(&self, bind: String, listener: TcpListener) {
        let next_server = self.spawn_server(bind, listener);
        let previous = {
            let mut active = self.active.lock().unwrap();
            active.replace(next_server)
        };

        if let Some(previous) = previous {
            previous.request_shutdown();
        }
    }

    fn spawn_server(&self, bind: String, listener: TcpListener) -> RunningStandaloneListenerServer {
        let generation = self.next_generation.fetch_add(1, Ordering::SeqCst);
        let (shutdown_tx, shutdown_rx) = oneshot::channel();
        let shutdown_requested = Arc::new(AtomicBool::new(false));
        let shutdown_requested_for_task = shutdown_requested.clone();
        let exit_tx = self.exit_tx.clone();
        let router = self.router.clone();
        let bind_for_task = bind.clone();
        let join_handle = tokio::spawn(async move {
            let result = axum::serve(listener, router)
                .with_graceful_shutdown(async move {
                    let _ = shutdown_rx.await;
                })
                .await;
            let _ = exit_tx.send(ListenerServerExit {
                generation,
                bind: bind_for_task,
                shutdown_requested: shutdown_requested_for_task.load(Ordering::SeqCst),
                result,
            });
        });

        RunningStandaloneListenerServer {
            bind,
            generation,
            shutdown_requested,
            shutdown_tx: Some(shutdown_tx),
            join_handle,
        }
    }

    fn current_bind(&self) -> Option<String> {
        self.active
            .lock()
            .unwrap()
            .as_ref()
            .map(|server| server.bind.clone())
    }

    fn take_active_server(&self) -> Option<RunningStandaloneListenerServer> {
        self.active.lock().unwrap().take()
    }
}

impl RunningStandaloneListenerServer {
    fn request_shutdown(mut self) {
        self.shutdown_requested.store(true, Ordering::SeqCst);
        if let Some(tx) = self.shutdown_tx.take() {
            let _ = tx.send(());
        }
        std::mem::drop(self.join_handle);
    }
}

impl PreparedStandaloneListenerRebind {
    pub(crate) fn activate(self) {
        self.inner.activate_prebound(self.bind, self.listener);
    }
}
