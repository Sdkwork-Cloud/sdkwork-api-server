use super::*;

const PORTAL_OPENAPI_DOCUMENT: &str = r##"{
  "openapi": "3.1.0",
  "info": {
    "title": "SDKWORK Portal API",
    "version": "0.1.0",
    "description": "OpenAPI 3.1 schema published from the current portal router surface."
  },
  "servers": [
    {
      "url": "/"
    }
  ],
  "tags": [
    {
      "name": "system",
      "description": "Portal health and system-facing routes."
    },
    {
      "name": "auth",
      "description": "Portal authentication and workspace access routes."
    },
    {
      "name": "marketing",
      "description": "Portal coupon validation, reservation, redemption, and reward history routes."
    },
    {
      "name": "billing",
      "description": "Portal billing account, ledger, and pricing visibility routes."
    },
    {
      "name": "jobs",
      "description": "Portal async job tracking routes."
    }
  ],
  "paths": {
    "/portal/health": {
      "get": {
        "tags": ["system"],
        "responses": {
          "200": {
            "description": "Portal health check response.",
            "content": {
              "text/plain": {
                "schema": {
                  "type": "string"
                }
              }
            }
          }
        }
      }
    },
    "/portal/auth/login": {
      "post": {
        "tags": ["auth"],
        "security": [],
        "requestBody": {
          "required": true,
          "content": {
            "application/json": {
              "schema": {
                "$ref": "#/components/schemas/LoginRequest"
              }
            }
          }
        },
        "responses": {
          "200": {
            "description": "Portal login session.",
            "content": {
              "application/json": {
                "schema": {
                  "$ref": "#/components/schemas/PortalAuthSession"
                }
              }
            }
          },
          "401": {
            "description": "Authentication failed.",
            "content": {
              "application/json": {
                "schema": {
                  "$ref": "#/components/schemas/ErrorResponse"
                }
              }
            }
          }
        }
      }
    },
    "/portal/workspace": {
      "get": {
        "tags": ["auth"],
        "security": [
          {
            "bearerAuth": []
          }
        ],
        "responses": {
          "200": {
            "description": "Current portal workspace summary.",
            "content": {
              "application/json": {
                "schema": {
                  "$ref": "#/components/schemas/PortalWorkspaceSummary"
                }
              }
            }
          },
          "401": {
            "description": "Portal authentication is required.",
            "content": {
              "application/json": {
                "schema": {
                  "$ref": "#/components/schemas/ErrorResponse"
                }
              }
            }
          }
        }
      }
    },
    "/portal/marketing/coupon-validations": {
      "post": {
        "tags": ["marketing"],
        "security": [
          {
            "bearerAuth": []
          }
        ],
        "requestBody": {
          "required": true,
          "content": {
            "application/json": {
              "schema": {
                "$ref": "#/components/schemas/PortalCouponValidationRequest"
              }
            }
          }
        },
        "responses": {
          "200": {
            "description": "Coupon validation result.",
            "content": {
              "application/json": {
                "schema": {
                  "$ref": "#/components/schemas/PortalCouponValidationResponse"
                }
              }
            }
          }
        }
      }
    },
    "/portal/marketing/coupon-reservations": {
      "post": {
        "tags": ["marketing"],
        "security": [
          {
            "bearerAuth": []
          }
        ],
        "requestBody": {
          "required": true,
          "content": {
            "application/json": {
              "schema": {
                "$ref": "#/components/schemas/PortalCouponReservationRequest"
              }
            }
          }
        },
        "responses": {
          "200": {
            "description": "Coupon reservation result.",
            "content": {
              "application/json": {
                "schema": {
                  "$ref": "#/components/schemas/PortalCouponReservationResponse"
                }
              }
            }
          }
        }
      }
    },
    "/portal/marketing/coupon-redemptions/confirm": {
      "post": {
        "tags": ["marketing"],
        "security": [
          {
            "bearerAuth": []
          }
        ],
        "requestBody": {
          "required": true,
          "content": {
            "application/json": {
              "schema": {
                "$ref": "#/components/schemas/PortalCouponRedemptionConfirmRequest"
              }
            }
          }
        },
        "responses": {
          "200": {
            "description": "Coupon redemption confirmation result.",
            "content": {
              "application/json": {
                "schema": {
                  "$ref": "#/components/schemas/PortalCouponRedemptionConfirmResponse"
                }
              }
            }
          }
        }
      }
    },
    "/portal/marketing/coupon-redemptions/rollback": {
      "post": {
        "tags": ["marketing"],
        "security": [
          {
            "bearerAuth": []
          }
        ],
        "requestBody": {
          "required": true,
          "content": {
            "application/json": {
              "schema": {
                "$ref": "#/components/schemas/PortalCouponRedemptionRollbackRequest"
              }
            }
          }
        },
        "responses": {
          "200": {
            "description": "Coupon redemption rollback result.",
            "content": {
              "application/json": {
                "schema": {
                  "$ref": "#/components/schemas/PortalCouponRedemptionRollbackResponse"
                }
              }
            }
          }
        }
      }
    },
    "/portal/marketing/my-coupons": {
      "get": {
        "tags": ["marketing"],
        "security": [
          {
            "bearerAuth": []
          }
        ],
        "responses": {
          "200": {
            "description": "Coupons visible to the current portal subject.",
            "content": {
              "application/json": {
                "schema": {
                  "$ref": "#/components/schemas/PortalMarketingCodesResponse"
                }
              }
            }
          }
        }
      }
    },
    "/portal/marketing/reward-history": {
      "get": {
        "tags": ["marketing"],
        "security": [
          {
            "bearerAuth": []
          }
        ],
        "responses": {
          "200": {
            "description": "Reward history for the current portal subject.",
            "content": {
              "application/json": {
                "schema": {
                  "type": "array",
                  "items": {
                    "$ref": "#/components/schemas/PortalMarketingRewardHistoryItem"
                  }
                }
              }
            }
          }
        }
      }
    },
    "/portal/marketing/redemptions": {
      "get": {
        "tags": ["marketing"],
        "security": [
          {
            "bearerAuth": []
          }
        ],
        "responses": {
          "200": {
            "description": "Coupon redemptions visible to the current portal subject.",
            "content": {
              "application/json": {
                "schema": {
                  "$ref": "#/components/schemas/PortalMarketingRedemptionsResponse"
                }
              }
            }
          }
        }
      }
    },
    "/portal/marketing/codes": {
      "get": {
        "tags": ["marketing"],
        "security": [
          {
            "bearerAuth": []
          }
        ],
        "responses": {
          "200": {
            "description": "Coupon codes visible to the current portal subject.",
            "content": {
              "application/json": {
                "schema": {
                  "$ref": "#/components/schemas/PortalMarketingCodesResponse"
                }
              }
            }
          }
        }
      }
    },
    "/portal/commerce/catalog": {
      "get": {
        "tags": ["commerce"],
        "security": [
          {
            "bearerAuth": []
          }
        ],
        "responses": {
          "200": {
            "description": "Canonical commerce catalog with compatibility views and explicit product and offer semantics.",
            "content": {
              "application/json": {
                "schema": {
                  "$ref": "#/components/schemas/PortalCommerceCatalog"
                }
              }
            }
          }
        }
      }
    },
    "/portal/commerce/orders/{order_id}": {
      "get": {
        "tags": ["commerce"],
        "security": [
          {
            "bearerAuth": []
          }
        ],
        "parameters": [
          {
            "name": "order_id",
            "in": "path",
            "required": true,
            "description": "Commerce order identifier.",
            "schema": {
              "type": "string"
            }
          }
        ],
        "responses": {
          "200": {
            "description": "Canonical order detail visible to the current workspace.",
            "content": {
              "application/json": {
                "schema": {
                  "$ref": "#/components/schemas/PortalCommerceOrder"
                }
              }
            }
          },
          "404": {
            "description": "Order not found or is not visible to the current portal subject.",
            "content": {
              "application/json": {
                "schema": {
                  "$ref": "#/components/schemas/ErrorResponse"
                }
              }
            }
          }
        }
      }
    },
    "/portal/commerce/orders/{order_id}/payment-methods": {
      "get": {
        "tags": ["commerce"],
        "security": [
          {
            "bearerAuth": []
          }
        ],
        "parameters": [
          {
            "name": "order_id",
            "in": "path",
            "required": true,
            "description": "Commerce order identifier.",
            "schema": {
              "type": "string"
            }
          }
        ],
        "responses": {
          "200": {
            "description": "Configured payment methods that are currently valid for the selected order.",
            "content": {
              "application/json": {
                "schema": {
                  "type": "array",
                  "items": {
                    "$ref": "#/components/schemas/PaymentMethodRecord"
                  }
                }
              }
            }
          },
          "404": {
            "description": "Order not found or is not visible to the current portal subject.",
            "content": {
              "application/json": {
                "schema": {
                  "$ref": "#/components/schemas/ErrorResponse"
                }
              }
            }
          }
        }
      }
    },
    "/portal/commerce/order-center": {
      "get": {
        "tags": ["commerce"],
        "security": [
          {
            "bearerAuth": []
          }
        ],
        "responses": {
          "200": {
            "description": "Aggregated order center view for the current workspace, including payment events and checkout status.",
            "content": {
              "application/json": {
                "schema": {
                  "$ref": "#/components/schemas/PortalCommerceOrderCenterResponse"
                }
              }
            }
          }
        }
      }
    },
    "/portal/commerce/payment-attempts/{payment_attempt_id}": {
      "get": {
        "tags": ["commerce"],
        "security": [
          {
            "bearerAuth": []
          }
        ],
        "parameters": [
          {
            "name": "payment_attempt_id",
            "in": "path",
            "required": true,
            "description": "Payment attempt identifier.",
            "schema": {
              "type": "string"
            }
          }
        ],
        "responses": {
          "200": {
            "description": "Canonical payment attempt detail for the current workspace.",
            "content": {
              "application/json": {
                "schema": {
                  "$ref": "#/components/schemas/CommercePaymentAttemptRecord"
                }
              }
            }
          },
          "404": {
            "description": "Payment attempt not found or is not visible to the current portal subject.",
            "content": {
              "application/json": {
                "schema": {
                  "$ref": "#/components/schemas/ErrorResponse"
                }
              }
            }
          }
        }
      }
    },
    "/portal/billing/account": {
      "get": {
        "tags": ["billing"],
        "security": [
          {
            "bearerAuth": []
          }
        ],
        "responses": {
          "200": {
            "description": "Commercial billing account and live balance snapshot for the current workspace.",
            "content": {
              "application/json": {
                "schema": {
                  "$ref": "#/components/schemas/PortalBillingAccountResponse"
                }
              }
            }
          },
          "401": {
            "description": "Portal authentication is required.",
            "content": {
              "application/json": {
                "schema": {
                  "$ref": "#/components/schemas/ErrorResponse"
                }
              }
            }
          },
          "501": {
            "description": "Commercial billing kernel is not configured.",
            "content": {
              "application/json": {
                "schema": {
                  "$ref": "#/components/schemas/ErrorResponse"
                }
              }
            }
          }
        }
      }
    },
    "/portal/billing/account-history": {
      "get": {
        "tags": ["billing"],
        "security": [
          {
            "bearerAuth": []
          }
        ],
        "responses": {
          "200": {
            "description": "Aggregated account history view for the current commercial billing account.",
            "content": {
              "application/json": {
                "schema": {
                  "$ref": "#/components/schemas/PortalBillingAccountHistoryResponse"
                }
              }
            }
          }
        }
      }
    },
    "/portal/billing/account/balance": {
      "get": {
        "tags": ["billing"],
        "security": [
          {
            "bearerAuth": []
          }
        ],
        "responses": {
          "200": {
            "description": "Current commercial billing balance snapshot.",
            "content": {
              "application/json": {
                "schema": {
                  "$ref": "#/components/schemas/AccountBalanceSnapshot"
                }
              }
            }
          }
        }
      }
    },
    "/portal/billing/account/benefit-lots": {
      "get": {
        "tags": ["billing"],
        "security": [
          {
            "bearerAuth": []
          }
        ],
        "responses": {
          "200": {
            "description": "Benefit lots attached to the current commercial billing account.",
            "content": {
              "application/json": {
                "schema": {
                  "type": "array",
                  "items": {
                    "$ref": "#/components/schemas/AccountBenefitLotRecord"
                  }
                }
              }
            }
          }
        }
      }
    },
    "/portal/billing/account/holds": {
      "get": {
        "tags": ["billing"],
        "security": [
          {
            "bearerAuth": []
          }
        ],
        "responses": {
          "200": {
            "description": "Current outstanding holds for the commercial billing account.",
            "content": {
              "application/json": {
                "schema": {
                  "type": "array",
                  "items": {
                    "$ref": "#/components/schemas/AccountHoldRecord"
                  }
                }
              }
            }
          }
        }
      }
    },
    "/portal/billing/account/request-settlements": {
      "get": {
        "tags": ["billing"],
        "security": [
          {
            "bearerAuth": []
          }
        ],
        "responses": {
          "200": {
            "description": "Request settlement history for the commercial billing account.",
            "content": {
              "application/json": {
                "schema": {
                  "type": "array",
                  "items": {
                    "$ref": "#/components/schemas/RequestSettlementRecord"
                  }
                }
              }
            }
          }
        }
      }
    },
    "/portal/billing/account/ledger": {
      "get": {
        "tags": ["billing"],
        "security": [
          {
            "bearerAuth": []
          }
        ],
        "responses": {
          "200": {
            "description": "Canonical ledger history for the commercial billing account.",
            "content": {
              "application/json": {
                "schema": {
                  "type": "array",
                  "items": {
                    "$ref": "#/components/schemas/AccountLedgerHistoryEntry"
                  }
                }
              }
            }
          }
        }
      }
    },
    "/portal/billing/pricing-plans": {
      "get": {
        "tags": ["billing"],
        "security": [
          {
            "bearerAuth": []
          }
        ],
        "responses": {
          "200": {
            "description": "Pricing plans visible to the current commercial billing account scope.",
            "content": {
              "application/json": {
                "schema": {
                  "type": "array",
                  "items": {
                    "$ref": "#/components/schemas/PricingPlanRecord"
                  }
                }
              }
            }
          }
        }
      }
    },
    "/portal/billing/pricing-rates": {
      "get": {
        "tags": ["billing"],
        "security": [
          {
            "bearerAuth": []
          }
        ],
        "responses": {
          "200": {
            "description": "Pricing rates visible to the current commercial billing account scope.",
            "content": {
              "application/json": {
                "schema": {
                  "type": "array",
                  "items": {
                    "$ref": "#/components/schemas/PricingRateRecord"
                  }
                }
              }
            }
          }
        }
      }
    },
    "/portal/async-jobs": {
      "get": {
        "tags": ["jobs"],
        "security": [
          {
            "bearerAuth": []
          }
        ],
        "responses": {
          "200": {
            "description": "Async jobs visible to the current portal subject.",
            "content": {
              "application/json": {
                "schema": {
                  "type": "array",
                  "items": {
                    "$ref": "#/components/schemas/AsyncJobRecord"
                  }
                }
              }
            }
          }
        }
      }
    },
    "/portal/async-jobs/{job_id}/attempts": {
      "get": {
        "tags": ["jobs"],
        "security": [
          {
            "bearerAuth": []
          }
        ],
        "parameters": [
          {
            "name": "job_id",
            "in": "path",
            "required": true,
            "description": "Async job identifier.",
            "schema": {
              "type": "string"
            }
          }
        ],
        "responses": {
          "200": {
            "description": "Attempts recorded for the selected async job.",
            "content": {
              "application/json": {
                "schema": {
                  "type": "array",
                  "items": {
                    "$ref": "#/components/schemas/AsyncJobAttemptRecord"
                  }
                }
              }
            }
          },
          "404": {
            "description": "Async job not found or is not visible to the current portal subject.",
            "content": {
              "application/json": {
                "schema": {
                  "$ref": "#/components/schemas/ErrorResponse"
                }
              }
            }
          }
        }
      }
    },
    "/portal/async-jobs/{job_id}/assets": {
      "get": {
        "tags": ["jobs"],
        "security": [
          {
            "bearerAuth": []
          }
        ],
        "parameters": [
          {
            "name": "job_id",
            "in": "path",
            "required": true,
            "description": "Async job identifier.",
            "schema": {
              "type": "string"
            }
          }
        ],
        "responses": {
          "200": {
            "description": "Assets recorded for the selected async job.",
            "content": {
              "application/json": {
                "schema": {
                  "type": "array",
                  "items": {
                    "$ref": "#/components/schemas/AsyncJobAssetRecord"
                  }
                }
              }
            }
          },
          "404": {
            "description": "Async job not found or is not visible to the current portal subject.",
            "content": {
              "application/json": {
                "schema": {
                  "$ref": "#/components/schemas/ErrorResponse"
                }
              }
            }
          }
        }
      }
    }
  },
  "components": {
    "securitySchemes": {
      "bearerAuth": {
        "type": "http",
        "scheme": "bearer",
        "bearerFormat": "JWT"
      }
    },
    "schemas": {
      "ErrorResponse": {
        "type": "object"
      },
      "LoginRequest": {
        "type": "object"
      },
      "PortalAuthSession": {
        "type": "object"
      },
      "PortalWorkspaceSummary": {
        "type": "object"
      },
      "PortalCouponValidationRequest": {
        "type": "object",
        "required": ["coupon_code", "subject_scope", "target_kind", "order_amount_minor", "reserve_amount_minor"],
        "properties": {
          "coupon_code": { "type": "string" },
          "subject_scope": { "type": "string" },
          "target_kind": { "type": "string" },
          "order_amount_minor": { "type": "integer", "format": "uint64", "minimum": 0 },
          "reserve_amount_minor": { "type": "integer", "format": "uint64", "minimum": 0 }
        }
      },
      "PortalCouponValidationResponse": {
        "type": "object"
      },
      "PortalCouponReservationRequest": {
        "type": "object",
        "required": ["coupon_code", "subject_scope", "target_kind", "reserve_amount_minor", "ttl_ms"],
        "properties": {
          "coupon_code": { "type": "string" },
          "subject_scope": { "type": "string" },
          "target_kind": { "type": "string" },
          "reserve_amount_minor": { "type": "integer", "format": "uint64", "minimum": 0 },
          "ttl_ms": { "type": "integer", "format": "uint64", "minimum": 0 },
          "idempotency_key": { "type": ["string", "null"] }
        }
      },
      "PortalCouponReservationResponse": {
        "type": "object"
      },
      "PortalCouponRedemptionConfirmRequest": {
        "type": "object"
      },
      "PortalCouponRedemptionConfirmResponse": {
        "type": "object"
      },
      "PortalCouponRedemptionRollbackRequest": {
        "type": "object"
      },
      "PortalCouponRedemptionRollbackResponse": {
        "type": "object"
      },
      "PortalCouponApplicabilitySummary": {
        "type": "object"
      },
      "PortalCouponEffectSummary": {
        "type": "object"
      },
      "PortalCouponOwnershipSummary": {
        "type": "object"
      },
      "PortalCouponAccountArrivalLotItem": {
        "type": "object",
        "properties": {
          "lot_id": {
            "type": "integer"
          },
          "benefit_type": {
            "type": "string"
          },
          "source_type": {
            "type": "string"
          },
          "source_id": {
            "type": "integer"
          },
          "status": {
            "type": "string"
          },
          "original_quantity": {
            "type": "number"
          },
          "remaining_quantity": {
            "type": "number"
          },
          "issued_at_ms": {
            "type": "integer"
          },
          "expires_at_ms": {
            "type": "integer"
          },
          "scope_order_id": {
            "type": "string"
          }
        }
      },
      "PortalCouponAccountArrivalSummary": {
        "type": "object",
        "properties": {
          "order_id": {
            "type": "string"
          },
          "account_id": {
            "type": "integer"
          },
          "benefit_lot_count": {
            "type": "integer"
          },
          "credited_quantity": {
            "type": "number"
          },
          "benefit_lots": {
            "type": "array",
            "items": {
              "$ref": "#/components/schemas/PortalCouponAccountArrivalLotItem"
            }
          }
        }
      },
      "CouponTemplateRecord": {
        "type": "object"
      },
      "MarketingCampaignRecord": {
        "type": "object"
      },
      "CouponCodeRecord": {
        "type": "object"
      },
      "CouponReservationRecord": {
        "type": "object"
      },
      "CouponRedemptionRecord": {
        "type": "object"
      },
      "CouponRollbackRecord": {
        "type": "object"
      },
      "PortalMarketingCodeItem": {
        "type": "object",
        "properties": {
          "code": {
            "$ref": "#/components/schemas/CouponCodeRecord"
          },
          "template": {
            "$ref": "#/components/schemas/CouponTemplateRecord"
          },
          "campaign": {
            "$ref": "#/components/schemas/MarketingCampaignRecord"
          },
          "applicability": {
            "$ref": "#/components/schemas/PortalCouponApplicabilitySummary"
          },
          "effect": {
            "$ref": "#/components/schemas/PortalCouponEffectSummary"
          },
          "ownership": {
            "$ref": "#/components/schemas/PortalCouponOwnershipSummary"
          },
          "latest_reservation": {
            "$ref": "#/components/schemas/CouponReservationRecord"
          },
          "latest_redemption": {
            "$ref": "#/components/schemas/CouponRedemptionRecord"
          }
        }
      },
      "PortalMarketingRewardHistoryItem": {
        "type": "object",
        "properties": {
          "redemption": {
            "$ref": "#/components/schemas/CouponRedemptionRecord"
          },
          "code": {
            "$ref": "#/components/schemas/CouponCodeRecord"
          },
          "template": {
            "$ref": "#/components/schemas/CouponTemplateRecord"
          },
          "campaign": {
            "$ref": "#/components/schemas/MarketingCampaignRecord"
          },
          "applicability": {
            "$ref": "#/components/schemas/PortalCouponApplicabilitySummary"
          },
          "effect": {
            "$ref": "#/components/schemas/PortalCouponEffectSummary"
          },
          "ownership": {
            "$ref": "#/components/schemas/PortalCouponOwnershipSummary"
          },
          "account_arrival": {
            "$ref": "#/components/schemas/PortalCouponAccountArrivalSummary"
          },
          "rollbacks": {
            "type": "array",
            "items": {
              "$ref": "#/components/schemas/CouponRollbackRecord"
            }
          }
        }
      },
      "PortalMarketingCodesResponse": {
        "type": "object",
        "properties": {
          "summary": {
            "type": "object"
          },
          "items": {
            "type": "array",
            "items": {
              "$ref": "#/components/schemas/PortalMarketingCodeItem"
            }
          }
        }
      },
      "PortalMarketingRedemptionsResponse": {
        "type": "object"
      },
      "PortalApiProduct": {
        "type": "object",
        "required": ["product_id", "product_kind", "target_id", "display_name", "source"],
        "properties": {
          "product_id": { "type": "string" },
          "product_kind": { "type": "string" },
          "target_id": { "type": "string" },
          "display_name": { "type": "string" },
          "source": { "type": "string" }
        }
      },
      "PortalProductOffer": {
        "type": "object",
        "required": ["offer_id", "product_id", "product_kind", "display_name", "quote_kind", "quote_target_kind", "quote_target_id", "source"],
        "properties": {
          "offer_id": { "type": "string" },
          "product_id": { "type": "string" },
          "product_kind": { "type": "string" },
          "display_name": { "type": "string" },
          "quote_kind": { "type": "string" },
          "quote_target_kind": { "type": "string" },
          "quote_target_id": { "type": "string" },
          "publication_id": { "type": "string" },
          "publication_kind": { "type": "string" },
          "publication_status": { "type": "string" },
          "publication_revision_id": { "type": "string" },
          "publication_version": { "type": "integer", "format": "uint64", "minimum": 0 },
          "publication_source_kind": { "type": "string" },
          "publication_effective_from_ms": { "type": ["integer", "null"], "format": "uint64", "minimum": 0 },
          "pricing_plan_id": { "type": ["string", "null"] },
          "pricing_plan_version": { "type": ["integer", "null"], "format": "uint64", "minimum": 0 },
          "pricing_rate_id": { "type": ["string", "null"] },
          "pricing_metric_code": { "type": ["string", "null"] },
          "price_label": { "type": ["string", "null"] },
          "source": { "type": "string" }
        }
      },
      "PortalCommerceCatalog": {
        "type": "object",
        "required": ["products", "offers", "plans", "packs", "recharge_options", "coupons"],
        "properties": {
          "products": {
            "type": "array",
            "items": { "$ref": "#/components/schemas/PortalApiProduct" }
          },
          "offers": {
            "type": "array",
            "items": { "$ref": "#/components/schemas/PortalProductOffer" }
          },
          "plans": {
            "type": "array",
            "items": { "type": "object" }
          },
          "packs": {
            "type": "array",
            "items": { "type": "object" }
          },
          "recharge_options": {
            "type": "array",
            "items": { "type": "object" }
          },
          "custom_recharge_policy": { "type": ["object", "null"] },
          "coupons": {
            "type": "array",
            "items": { "type": "object" }
          }
        }
      },
      "PortalCommerceOrder": {
        "type": "object",
        "required": ["order_id", "project_id", "user_id", "target_kind", "target_id", "target_name", "transaction_kind", "status"],
        "properties": {
          "order_id": { "type": "string" },
          "project_id": { "type": "string" },
          "user_id": { "type": "string" },
          "target_kind": { "type": "string" },
          "product_kind": { "type": "string" },
          "transaction_kind": { "type": "string" },
          "product_id": { "type": "string" },
          "offer_id": { "type": "string" },
          "publication_id": { "type": "string" },
          "publication_kind": { "type": "string" },
          "publication_status": { "type": "string" },
          "publication_revision_id": { "type": "string" },
          "publication_version": { "type": "integer", "format": "uint64", "minimum": 0 },
          "publication_source_kind": { "type": "string" },
          "publication_effective_from_ms": { "type": ["integer", "null"], "format": "uint64", "minimum": 0 },
          "target_id": { "type": "string" },
          "target_name": { "type": "string" },
          "pricing_plan_id": { "type": ["string", "null"] },
          "pricing_plan_version": { "type": ["integer", "null"] },
          "pricing_rate_id": { "type": ["string", "null"] },
          "pricing_metric_code": { "type": ["string", "null"] },
          "payment_method_id": { "type": ["string", "null"] },
          "latest_payment_attempt_id": { "type": ["string", "null"] },
          "status": { "type": "string" }
        }
      },
      "PortalCommerceOrderCenterResponse": {
        "type": "object"
      },
      "PaymentMethodRecord": {
        "type": "object",
        "required": ["payment_method_id", "display_name", "provider", "channel", "enabled"],
        "properties": {
          "payment_method_id": { "type": "string" },
          "display_name": { "type": "string" },
          "provider": { "type": "string" },
          "channel": { "type": "string" },
          "enabled": { "type": "boolean" },
          "supported_currency_codes": {
            "type": "array",
            "items": { "type": "string" }
          },
          "supported_order_kinds": {
            "type": "array",
            "items": { "type": "string" }
          }
        }
      },
      "CommercePaymentAttemptRecord": {
        "type": "object",
        "required": ["payment_attempt_id", "order_id", "payment_method_id", "provider", "channel", "status"],
        "properties": {
          "payment_attempt_id": { "type": "string" },
          "order_id": { "type": "string" },
          "payment_method_id": { "type": "string" },
          "provider": { "type": "string" },
          "channel": { "type": "string" },
          "status": { "type": "string" },
          "provider_payment_intent_id": { "type": ["string", "null"] },
          "provider_checkout_session_id": { "type": ["string", "null"] },
          "provider_reference": { "type": ["string", "null"] },
          "checkout_url": { "type": ["string", "null"] }
        }
      },
      "PortalBillingAccountHistoryResponse": {
        "type": "object"
      },
      "PortalBillingAccountResponse": {
        "type": "object"
      },
      "AccountBalanceSnapshot": {
        "type": "object"
      },
      "AccountBenefitLotRecord": {
        "type": "object"
      },
      "AccountHoldRecord": {
        "type": "object"
      },
      "RequestSettlementRecord": {
        "type": "object"
      },
      "AccountLedgerHistoryEntry": {
        "type": "object"
      },
      "PricingPlanRecord": {
        "type": "object"
      },
      "PricingRateRecord": {
        "type": "object"
      },
      "AsyncJobRecord": {
        "type": "object"
      },
      "AsyncJobAttemptRecord": {
        "type": "object"
      },
      "AsyncJobAssetRecord": {
        "type": "object"
      }
    }
  }
}"##;

async fn portal_openapi_handler() -> impl axum::response::IntoResponse {
    (
        [(header::CONTENT_TYPE, "application/json; charset=utf-8")],
        PORTAL_OPENAPI_DOCUMENT,
    )
}

async fn portal_docs_index_handler() -> Html<String> {
    Html(
        r#"<!doctype html>
<html lang="en">
  <head>
    <meta charset="utf-8" />
    <meta name="viewport" content="width=device-width, initial-scale=1" />
    <title>SDKWORK Portal API</title>
    <style>
      :root {
        color-scheme: light;
        font-family: "Segoe UI", "PingFang SC", sans-serif;
      }
      body {
        margin: 0;
        background: linear-gradient(180deg, #f5f7fb 0%, #eef2f8 100%);
        color: #132238;
      }
      main {
        max-width: 960px;
        margin: 0 auto;
        padding: 48px 24px 64px;
      }
      .card {
        background: rgba(255, 255, 255, 0.92);
        border: 1px solid rgba(19, 34, 56, 0.08);
        border-radius: 20px;
        box-shadow: 0 20px 60px rgba(19, 34, 56, 0.08);
        padding: 32px;
      }
      code {
        background: rgba(19, 34, 56, 0.08);
        border-radius: 8px;
        padding: 2px 8px;
      }
      a {
        color: #0f6ab4;
        text-decoration: none;
      }
      a:hover {
        text-decoration: underline;
      }
    </style>
  </head>
  <body>
    <main>
      <section class="card">
        <p>OpenAPI 3.1</p>
        <h1>SDKWORK Portal API</h1>
        <p>The live contract for the current portal surface is published at <code>/portal/openapi.json</code>.</p>
        <p><a href="/portal/openapi.json">Open the raw schema</a></p>
      </section>
    </main>
  </body>
</html>"#
            .to_string(),
    )
}

pub(crate) fn portal_docs_router<S>() -> Router<S>
where
    S: Clone + Send + Sync + 'static,
{
    Router::new()
        .route("/portal/openapi.json", get(portal_openapi_handler))
        .route("/portal/docs", get(portal_docs_index_handler))
}
