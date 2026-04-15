mod errors;
mod mode;
mod normalize;
mod relations;

pub(crate) use errors::{marketing_create_invalid_input, marketing_create_storage};
pub(crate) use mode::PersistMode;
pub(crate) use normalize::{
    normalize_optional_identifier, normalize_optional_text, normalize_required_identifier,
};
pub(crate) use relations::{
    load_coupon_template_record, require_marketing_campaign_record,
    validate_coupon_code_template_compatibility,
};
