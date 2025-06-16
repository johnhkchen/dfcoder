use crate::*;
use reqwest::Client;
use serde_json::json;
use std::time::Duration;

/// BAML client for communicating with language models
#[derive(Debug, Clone)]
pub struct BamlClient {
    client: Client,
    config: BamlConfig,
}

impl BamlClient {
    /// Create a new BAML client
    pub fn new(config: BamlConfig) -> Result<Self, BamlError> {
        let client = Client::builder()
            .timeout(Duration::from_secs(30))
            .build()
            .map_err(|e| BamlError::ClientError(format!("Failed to create HTTP client: {}", e)))?;
        
        Ok(Self { client, config })
    }
    
    /// Generate a response from the language model
    pub async fn generate_response(&self, prompt: &str) -> Result<String, BamlError> {
        let request_body = json!({
            "model": self.config.model,
            "max_tokens": self.config.max_tokens,
            "temperature": self.config.temperature,
            "messages": [
                {
                    "role": "user",
                    "content": prompt
                }
            ]
        });
        
        let mut request = self.client
            .post(&self.config.endpoint)
            .json(&request_body);
        
        if let Some(api_key) = &self.config.api_key {
            request = request.header("x-api-key", api_key);
        }
        
        let response = request
            .send()
            .await
            .map_err(|e| BamlError::ClientError(format!("Request failed: {}", e)))?;
        
        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().await.unwrap_or_else(|_| "Unknown error".to_string());
            return Err(BamlError::ApiError(format!("API error {}: {}", status, error_text)));
        }
        
        let response_json: serde_json::Value = response
            .json()
            .await
            .map_err(|e| BamlError::JsonError(e))?;
        
        self.extract_content_from_response(&response_json)
    }
    
    /// Classify text using a structured prompt
    pub async fn classify(&self, text: &str, categories: &[&str]) -> Result<ClassificationResponse, BamlError> {
        let prompt = self.build_classification_prompt(text, categories);
        let response = self.generate_response(&prompt).await?;
        self.parse_classification_response(&response)
    }
    
    /// Batch classify multiple texts
    pub async fn classify_batch(&self, texts: &[&str], categories: &[&str]) -> Result<Vec<ClassificationResponse>, BamlError> {
        let mut results = Vec::new();
        
        // For now, process sequentially. In production, this could be parallelized
        for text in texts {
            let result = self.classify(text, categories).await?;
            results.push(result);
        }
        
        Ok(results)
    }
    
    /// Extract structured data using a schema
    pub async fn extract_structured_data<T>(&self, text: &str, schema: &str) -> Result<T, BamlError>
    where
        T: for<'de> serde::Deserialize<'de>,
    {
        let prompt = format!(
            "Extract structured data from the following text according to this schema:\n\n\
            Schema: {}\n\n\
            Text: {}\n\n\
            Return the result as valid JSON that matches the schema.",
            schema, text
        );
        
        let response = self.generate_response(&prompt).await?;
        serde_json::from_str(&response)
            .map_err(|e| BamlError::JsonError(e))
    }
    
    /// Analyze sentiment and intent
    pub async fn analyze_sentiment(&self, text: &str) -> Result<SentimentAnalysis, BamlError> {
        let prompt = format!(
            "Analyze the sentiment and intent of this text:\n\n\
            Text: {}\n\n\
            Return analysis as JSON with fields: sentiment (positive/negative/neutral), \
            confidence (0-1), intent (question/request/statement/complaint), and reasoning.",
            text
        );
        
        let response = self.generate_response(&prompt).await?;
        serde_json::from_str(&response)
            .map_err(|e| BamlError::JsonError(e))
    }
    
    fn build_classification_prompt(&self, text: &str, categories: &[&str]) -> String {
        format!(
            "Classify the following text into one of these categories: {}\n\n\
            Text: {}\n\n\
            Return your response as JSON with fields:\n\
            - category: the most appropriate category\n\
            - confidence: a number between 0 and 1\n\
            - reasoning: brief explanation for the classification\n\
            - indicators: list of keywords or phrases that influenced the decision",
            categories.join(", "),
            text
        )
    }
    
    fn parse_classification_response(&self, response: &str) -> Result<ClassificationResponse, BamlError> {
        // Try to extract JSON from the response
        let json_start = response.find('{');
        let json_end = response.rfind('}');
        
        match (json_start, json_end) {
            (Some(start), Some(end)) if start < end => {
                let json_str = &response[start..=end];
                serde_json::from_str(json_str)
                    .map_err(|e| BamlError::JsonError(e))
            }
            _ => {
                // Fallback: try to parse the entire response
                serde_json::from_str(response)
                    .map_err(|e| BamlError::JsonError(e))
            }
        }
    }
    
    fn extract_content_from_response(&self, response: &serde_json::Value) -> Result<String, BamlError> {
        // Handle different API response formats
        if let Some(content) = response["content"][0]["text"].as_str() {
            Ok(content.to_string())
        } else if let Some(content) = response["choices"][0]["message"]["content"].as_str() {
            Ok(content.to_string())
        } else if let Some(content) = response["text"].as_str() {
            Ok(content.to_string())
        } else {
            Err(BamlError::ApiError("Unable to extract content from response".to_string()))
        }
    }
}

/// Response from text classification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClassificationResponse {
    pub category: String,
    pub confidence: f32,
    pub reasoning: String,
    pub indicators: Vec<String>,
}

/// Sentiment analysis results
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SentimentAnalysis {
    pub sentiment: String,
    pub confidence: f32,
    pub intent: String,
    pub reasoning: String,
}

/// Error types for BAML operations
#[derive(Debug, thiserror::Error)]
pub enum BamlError {
    #[error("BAML client error: {0}")]
    ClientError(String),
    #[error("API error: {0}")]
    ApiError(String),
    #[error("JSON parsing error: {0}")]
    JsonError(#[from] serde_json::Error),
    #[error("Configuration error: {0}")]
    ConfigError(String),
    #[error("Classification error: {0}")]
    ClassificationError(String),
}

/// Builder for BAML client configuration
pub struct BamlClientBuilder {
    config: BamlConfig,
}

impl BamlClientBuilder {
    pub fn new() -> Self {
        Self {
            config: BamlConfig::default(),
        }
    }
    
    pub fn endpoint(mut self, endpoint: impl Into<String>) -> Self {
        self.config.endpoint = endpoint.into();
        self
    }
    
    pub fn api_key(mut self, api_key: impl Into<String>) -> Self {
        self.config.api_key = Some(api_key.into());
        self
    }
    
    pub fn model(mut self, model: impl Into<String>) -> Self {
        self.config.model = model.into();
        self
    }
    
    pub fn temperature(mut self, temperature: f32) -> Self {
        self.config.temperature = temperature;
        self
    }
    
    pub fn max_tokens(mut self, max_tokens: u32) -> Self {
        self.config.max_tokens = max_tokens;
        self
    }
    
    pub fn confidence_threshold(mut self, threshold: f32) -> Self {
        self.config.confidence_threshold = threshold;
        self
    }
    
    pub fn build(self) -> Result<BamlClient, BamlError> {
        BamlClient::new(self.config)
    }
}

impl Default for BamlClientBuilder {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_client_creation() {
        let config = BamlConfig::default();
        let client = BamlClient::new(config);
        assert!(client.is_ok());
    }
    
    #[test]
    fn test_classification_prompt_building() {
        let config = BamlConfig::default();
        let client = BamlClient::new(config).unwrap();
        let categories = ["positive", "negative", "neutral"];
        let prompt = client.build_classification_prompt("I love this!", &categories);
        assert!(prompt.contains("positive"));
        assert!(prompt.contains("I love this!"));
    }
}