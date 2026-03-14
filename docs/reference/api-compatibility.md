# API Compatibility

The gateway uses five execution-truth labels:

- `native`
- `relay`
- `translated`
- `emulated`
- `unsupported`

## High-Value API Families

Currently implemented gateway families include:

- `/v1/models`
- `/v1/chat/completions`
- `/v1/completions`
- `/v1/responses`
- `/v1/embeddings`
- `/v1/files`
- `/v1/uploads`
- `/v1/audio/*`
- `/v1/images/*`
- `/v1/assistants`
- `/v1/threads`
- `/v1/conversations`
- `/v1/vector_stores`
- `/v1/batches`
- `/v1/fine_tuning/jobs`
- `/v1/webhooks`
- `/v1/evals`
- `/v1/videos`

The control plane also exposes:

- `/admin/*`
- `/portal/*`

## Detailed References

Read the full data-plane and control-plane matrix here:

- [Full Compatibility Matrix](/api/compatibility-matrix)
