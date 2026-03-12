use sdkwork_api_contract_openai::embeddings::CreateEmbeddingResponse;

#[test]
fn serializes_embeddings_list() {
    let response = CreateEmbeddingResponse::empty("text-embedding-3-large");
    assert_eq!(response.object, "list");
}
