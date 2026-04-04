// src/stdlib/common/fastembed_provider.rs

use easy_error::{Error, bail};
use fastembed::{EmbeddingModel, InitOptions, TextEmbedding};
use std::sync::Mutex;

use deepthought::deepthought_vector::EmbeddingProvider;

/// FastEmbed provider for generating embeddings locally
/// Uses the fastembed-rs library for efficient, local embedding generation
/// Synchronous implementation - no async runtime required
pub struct FastEmbedProvider {
    model: Mutex<TextEmbedding>, // Use Mutex for interior mutability
    model_name: String,
    batch_size: usize,
}

impl FastEmbedProvider {
    /// Create a new FastEmbed provider with default settings (all-MiniLM-L6-v2)
    pub fn new() -> Result<Self, Error> {
        Self::with_model(EmbeddingModel::AllMiniLML6V2, 256)
    }

    /// Create a new FastEmbed provider with a specific model
    pub fn with_model(model: EmbeddingModel, batch_size: usize) -> Result<Self, Error> {
        // Store model name before moving it
        let model_name = format!("{:?}", model);

        // Use InitOptions::new() which is the current API
        let init_options = InitOptions::new(model);

        let embedding_model = match TextEmbedding::try_new(init_options) {
            Ok(model) => model,
            Err(e) => bail!("Failed to initialize FastEmbed model: {:?}", e),
        };

        Ok(Self {
            model: Mutex::new(embedding_model),
            model_name,
            batch_size,
        })
    }

    /// Create a new FastEmbed provider with custom initialization options
    pub fn with_options(options: InitOptions, batch_size: usize) -> Result<Self, Error> {
        // Get model name from options if possible, otherwise use default
        let model_name = format!("{:?}", options.model_name);

        let embedding_model = match TextEmbedding::try_new(options) {
            Ok(model) => model,
            Err(e) => bail!(
                "Failed to initialize FastEmbed model with custom options: {:?}",
                e
            ),
        };

        Ok(Self {
            model: Mutex::new(embedding_model),
            model_name,
            batch_size,
        })
    }

    /// Get the dimension of the embedding vectors for the current model
    /// Get dimension by generating a sample embedding
    pub fn embedding_dimension(&self) -> Result<usize, Error> {
        // Generate a sample embedding to get the dimension
        let sample = self.generate_embedding("test", "")?;
        Ok(sample.len())
    }

    /// Get the name of the current model
    pub fn model_name(&self) -> &str {
        &self.model_name
    }

    /// Get the batch size being used
    pub fn batch_size(&self) -> usize {
        self.batch_size
    }
}

impl EmbeddingProvider for FastEmbedProvider {
    fn generate_embedding(&self, text: &str, prefix: &str) -> Result<Vec<f32>, Error> {
        let prefixed_text = if prefix.is_empty() {
            text.to_string()
        } else {
            format!("{} {}", prefix, text)
        };

        // FastEmbed expects a vector of strings
        let documents = vec![prefixed_text];

        // Lock the mutex to get mutable access to the model
        let mut model = match self.model.lock() {
            Ok(model) => model,
            Err(e) => bail!("Failed to lock FastEmbed model: {}", e),
        };

        match model.embed(documents, None) {
            Ok(embeddings) => {
                if embeddings.is_empty() {
                    bail!("No embedding generated for text: {}", text);
                }
                // FastEmbed returns Vec<Vec<f32>>, we need the first (and only) embedding
                Ok(embeddings[0].clone())
            }
            Err(e) => bail!("Failed to generate embedding with FastEmbed: {:?}", e),
        }
    }

    fn generate_batch_embeddings(
        &self,
        texts: &[String],
        prefix: &str,
    ) -> Result<Vec<Vec<f32>>, Error> {
        if texts.is_empty() {
            return Ok(Vec::new());
        }

        // Apply prefix to all texts
        let prefixed_texts: Vec<String> = if prefix.is_empty() {
            texts.to_vec()
        } else {
            texts
                .iter()
                .map(|text| format!("{} {}", prefix, text))
                .collect()
        };

        // Lock the mutex once for the entire batch operation
        let mut model = match self.model.lock() {
            Ok(model) => model,
            Err(e) => bail!("Failed to lock FastEmbed model: {}", e),
        };

        // Process in batches to avoid memory issues
        let mut all_embeddings = Vec::with_capacity(prefixed_texts.len());

        for chunk in prefixed_texts.chunks(self.batch_size) {
            match model.embed(chunk.to_vec(), None) {
                Ok(mut embeddings) => {
                    all_embeddings.append(&mut embeddings);
                }
                Err(e) => bail!(
                    "Failed to generate batch embeddings with FastEmbed: {:?}",
                    e
                ),
            }
        }

        if all_embeddings.len() != texts.len() {
            bail!(
                "Embedding count mismatch: expected {}, got {}",
                texts.len(),
                all_embeddings.len()
            );
        }

        Ok(all_embeddings)
    }
}

// ============================================
// Available FastEmbed Models
// ============================================

/// Convenience methods for creating FastEmbed providers with specific models
impl FastEmbedProvider {
    /// BAAI/bge-small-en-v1.5 - Good balance of quality and speed (384-dim)
    pub fn bge_small() -> Result<Self, Error> {
        Self::with_model(EmbeddingModel::BGESmallENV15, 256)
    }

    /// BAAI/bge-base-en-v1.5 - Higher quality, slower (768-dim)
    pub fn bge_base() -> Result<Self, Error> {
        Self::with_model(EmbeddingModel::BGEBaseENV15, 128)
    }

    /// BAAI/bge-large-en-v1.5 - Best quality, slowest (1024-dim)
    pub fn bge_large() -> Result<Self, Error> {
        Self::with_model(EmbeddingModel::BGELargeENV15, 64)
    }

    /// sentence-transformers/all-MiniLM-L6-v2 - Fast, small, good for general use (384-dim)
    pub fn mini_lm() -> Result<Self, Error> {
        Self::with_model(EmbeddingModel::AllMiniLML6V2, 256)
    }

    /// sentence-transformers/all-mpnet-base-v2 - High quality, slower (768-dim)
    pub fn mpnet() -> Result<Self, Error> {
        Self::with_model(EmbeddingModel::AllMpnetBaseV2, 128)
    }
}

// ============================================
// Lazy initialization helper for global/shared use
// ============================================

use std::sync::OnceLock;

/// Global FastEmbed provider singleton (useful for sharing across threads)
static GLOBAL_FAST_EMBED: OnceLock<FastEmbedProvider> = OnceLock::new();

/// Initialize or get the global FastEmbed provider
pub fn get_global_fast_embed() -> Result<&'static FastEmbedProvider, Error> {
    match GLOBAL_FAST_EMBED.get() {
        Some(provider) => Ok(provider),
        None => {
            let provider = FastEmbedProvider::mini_lm()?;
            match GLOBAL_FAST_EMBED.set(provider) {
                Ok(_) => Ok(GLOBAL_FAST_EMBED.get().unwrap()),
                Err(_) => bail!("Failed to set global FastEmbed provider"),
            }
        }
    }
}

/// Initialize the global FastEmbed provider with a specific model
pub fn init_global_fast_embed(model: EmbeddingModel, batch_size: usize) -> Result<(), Error> {
    let provider = FastEmbedProvider::with_model(model, batch_size)?;
    match GLOBAL_FAST_EMBED.set(provider) {
        Ok(_) => Ok(()),
        Err(_) => bail!("Global FastEmbed provider already initialized"),
    }
}

/// Reset the global FastEmbed provider (useful for testing)
#[cfg(test)]
pub fn reset_global_fast_embed() {
    let _ = GLOBAL_FAST_EMBED.take();
}

// ============================================
// Utility Functions
// ============================================

/// Calculate cosine similarity between two embeddings
pub fn cosine_similarity(vec1: &[f32], vec2: &[f32]) -> Result<f32, Error> {
    if vec1.len() != vec2.len() {
        bail!(
            "Vector dimension mismatch: {} vs {}",
            vec1.len(),
            vec2.len()
        );
    }

    let dot_product: f32 = vec1.iter().zip(vec2.iter()).map(|(a, b)| a * b).sum();
    let norm1: f32 = vec1.iter().map(|x| x * x).sum::<f32>().sqrt();
    let norm2: f32 = vec2.iter().map(|x| x * x).sum::<f32>().sqrt();

    if norm1 == 0.0 || norm2 == 0.0 {
        bail!("Zero norm vector detected");
    }

    Ok(dot_product / (norm1 * norm2))
}

/// Normalize an embedding vector to unit length
pub fn normalize_embedding(vec: &mut [f32]) {
    let norm: f32 = vec.iter().map(|x| x * x).sum::<f32>().sqrt();
    if norm > 0.0 {
        for x in vec.iter_mut() {
            *x /= norm;
        }
    }
}

/// Get recommended batch size based on model
pub fn recommended_batch_size(model: &EmbeddingModel) -> usize {
    match model {
        EmbeddingModel::AllMiniLML6V2 => 256,
        EmbeddingModel::AllMpnetBaseV2 => 128,
        EmbeddingModel::BGESmallENV15 => 256,
        EmbeddingModel::BGEBaseENV15 => 128,
        EmbeddingModel::BGELargeENV15 => 64,
        _ => 128, // Default fallback
    }
}

// ============================================
// Tests
// ============================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fastembed_creation() {
        let provider = FastEmbedProvider::new();
        assert!(provider.is_ok());

        let provider = provider.unwrap();
        assert!(provider.model_name().contains("AllMiniLML6V2"));
        assert!(provider.embedding_dimension().unwrap() > 0);
    }

    #[test]
    fn test_fastembed_generate_embedding() {
        let provider = FastEmbedProvider::new().unwrap();
        let embedding = provider.generate_embedding("Hello world", "query: ");

        assert!(embedding.is_ok());
        let vec = embedding.unwrap();
        assert_eq!(vec.len(), provider.embedding_dimension().unwrap());
    }

    #[test]
    fn test_fastembed_batch_embeddings() {
        let provider = FastEmbedProvider::new().unwrap();
        let texts = vec![
            "First document".to_string(),
            "Second document".to_string(),
            "Third document".to_string(),
        ];

        let embeddings = provider.generate_batch_embeddings(&texts, "");
        assert!(embeddings.is_ok());

        let embeddings = embeddings.unwrap();
        assert_eq!(embeddings.len(), 3);
        assert_eq!(embeddings[0].len(), provider.embedding_dimension().unwrap());
    }

    #[test]
    fn test_fastembed_with_prefix() {
        let provider = FastEmbedProvider::new().unwrap();
        let embedding_without = provider.generate_embedding("test", "");
        let embedding_with = provider.generate_embedding("test", "query: ");

        assert!(embedding_without.is_ok());
        assert!(embedding_with.is_ok());

        // They should be different because of the prefix
        assert_ne!(embedding_without.unwrap(), embedding_with.unwrap());
    }

    #[test]
    fn test_fastembed_different_models() {
        let mini_lm = FastEmbedProvider::mini_lm();
        let bge_small = FastEmbedProvider::bge_small();

        assert!(mini_lm.is_ok());
        assert!(bge_small.is_ok());

        // Different models may have different dimensions
        let dim1 = mini_lm.unwrap().embedding_dimension().unwrap();
        let dim2 = bge_small.unwrap().embedding_dimension().unwrap();

        // Both should have valid dimensions
        assert!(dim1 > 0);
        assert!(dim2 > 0);
    }

    #[test]
    fn test_global_fast_embed() {
        // Reset any existing global
        reset_global_fast_embed();

        let provider = get_global_fast_embed();
        assert!(provider.is_ok());

        let embedding = provider.unwrap().generate_embedding("test", "");
        assert!(embedding.is_ok());
    }

    #[test]
    fn test_cosine_similarity() {
        let vec1 = vec![1.0, 0.0, 0.0];
        let vec2 = vec![1.0, 0.0, 0.0];

        let similarity = cosine_similarity(&vec1, &vec2).unwrap();
        assert!((similarity - 1.0).abs() < 0.0001);

        let vec3 = vec![0.0, 1.0, 0.0];
        let similarity = cosine_similarity(&vec1, &vec3).unwrap();
        assert!((similarity - 0.0).abs() < 0.0001);
    }

    #[test]
    fn test_normalize_embedding() {
        let mut vec = vec![3.0, 4.0];
        normalize_embedding(&mut vec);

        let norm: f32 = vec.iter().map(|x| x * x).sum::<f32>().sqrt();
        assert!((norm - 1.0).abs() < 0.0001);
    }
}
