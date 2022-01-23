#[async_trait::async_trait]
pub trait ReadChunks {
    async fn read_chunks(&mut self) -> crate::Result<Vec<u8>>;
}

#[async_trait::async_trait]
impl ReadChunks for hyper::Body {
    async fn read_chunks(&mut self) -> crate::Result<Vec<u8>> {
        let mut chunks = Vec::new();

        while let Some(chunk) = hyper::body::HttpBody::data(self).await {
            let chunk = chunk.map_err(crate::Error::ReadChunksFromBody)?;

            chunks.push(chunk);
        }

        Ok(chunks.concat())
    }
}
