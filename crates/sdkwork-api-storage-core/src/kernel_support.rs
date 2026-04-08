use super::*;

pub(crate) fn unsupported_account_kernel_method(
    dialect: StorageDialect,
    method: &str,
) -> anyhow::Error {
    anyhow::anyhow!(
        "storage dialect {} does not implement canonical account kernel method {} yet",
        dialect.as_str(),
        method
    )
}

pub(crate) fn unsupported_identity_kernel_method(
    dialect: StorageDialect,
    method: &str,
) -> anyhow::Error {
    anyhow::anyhow!(
        "storage dialect {} does not implement canonical identity kernel method {} yet",
        dialect.as_str(),
        method
    )
}

pub(crate) fn unsupported_marketing_kernel_method(
    dialect: StorageDialect,
    method: &str,
) -> anyhow::Error {
    anyhow::anyhow!(
        "storage dialect {} does not implement enterprise marketing kernel method {} yet",
        dialect.as_str(),
        method
    )
}

pub(crate) fn unsupported_commerce_method(dialect: StorageDialect, method: &str) -> anyhow::Error {
    anyhow::anyhow!(
        "storage dialect {} does not implement commerce method {} yet",
        dialect.as_str(),
        method
    )
}
