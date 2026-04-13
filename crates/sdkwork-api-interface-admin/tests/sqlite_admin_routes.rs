use serial_test::serial;

#[serial]
mod sqlite_admin_routes {
    mod support {
        include!("sqlite_admin_routes/support.rs");
    }

    mod api_keys_groups {
        include!("sqlite_admin_routes/api_keys_groups.rs");
    }

    mod auth_catalog_users {
        include!("sqlite_admin_routes/auth_catalog_users.rs");
    }

    mod cluster_billing {
        include!("sqlite_admin_routes/cluster_billing.rs");
    }

    mod extensions_runtime {
        include!("sqlite_admin_routes/extensions_runtime.rs");
    }

    mod provider_accounts {
        include!("sqlite_admin_routes/provider_accounts.rs");
    }

    mod providers_models_coupons {
        include!("sqlite_admin_routes/providers_models_coupons.rs");
    }

    mod routing {
        include!("sqlite_admin_routes/routing.rs");
    }
}
