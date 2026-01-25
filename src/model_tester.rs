use anyhow::{Result, Context};
use serde::{Deserialize, Serialize};
use std::time::{Duration, Instant};
use crate::ollama_client::{OllamaClient, OllamaOptions};
use crate::prompt_builder::PromptBuilder;
use crate::rag_validator::{RAGValidator, ValidationResult};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelTestResult {
    pub model_name: String,
    pub query: String,
    pub response: String,
    pub response_time_ms: u128,
    pub tokens_per_second: Option<f32>,
    pub validation_result: Option<ValidationResult>,
    pub accuracy_score: f32,
    pub quality_score: f32,
    pub timestamp: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelComparison {
    pub models_tested: Vec<String>,
    pub test_queries: Vec<String>,
    pub results: Vec<ModelTestResult>,
    pub summary: ComparisonSummary,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComparisonSummary {
    pub best_accuracy: String,
    pub best_speed: String,
    pub best_quality: String,
    pub recommended_model: String,
    pub reasoning: String,
}

#[derive(Debug, Clone)]
pub struct TestQuery {
    pub query: String,
    pub category: QueryCategory,
    pub expected_keywords: Vec<String>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum QueryCategory {
    Efun,
    ObjectSystem,
    Combat,
    Codegen,
    GeneralLPC,
}

pub struct ModelTester {
    ollama_client: OllamaClient,
    prompt_builder: PromptBuilder,
    validator: Option<RAGValidator>,
}

impl ModelTester {
    pub fn new(
        ollama_client: OllamaClient,
        prompt_builder: PromptBuilder,
        validator: Option<RAGValidator>,
    ) -> Self {
        Self {
            ollama_client,
            prompt_builder,
            validator,
        }
    }

    /// Run comprehensive model comparison test
    pub async fn compare_models(&self, models: Vec<String>, queries: Vec<TestQuery>) -> Result<ModelComparison> {
        let mut results = Vec::new();
        
        for model in &models {
            println!("Testing model: {}", model);
            
            for query in &queries {
                println!("  Running query: {}", query.query);
                
                match self.test_model(model, query).await {
                    Ok(result) => {
                        println!("    ✓ Completed in {}ms", result.response_time_ms);
                        results.push(result);
                    }
                    Err(e) => {
                        eprintln!("    ✗ Error: {}", e);
                        // Continue with other tests
                    }
                }
            }
        }
        
        let summary = self.generate_summary(&models, &results);
        
        Ok(ModelComparison {
            models_tested: models,
            test_queries: queries.iter().map(|q| q.query.clone()).collect(),
            results,
            summary,
        })
    }

    /// Test a single model with a single query
    pub async fn test_model(&self, model_name: &str, query: &TestQuery) -> Result<ModelTestResult> {
        let start = Instant::now();
        
        // Build prompt with RAG context
        let validation_result = if let Some(validator) = &self.validator {
            Some(validator.validate_query(&query.query)?)
        } else {
            None
        };
        
        let prompt = self.prompt_builder.build_prompt(&query.query, model_name, Vec::new())
            .map_err(|e| anyhow::anyhow!("Failed to build prompt: {}", e))?;
        
        // Configure options for testing
        let options = OllamaOptions::with_defaults(
            Some(0.3),  // Low temperature for consistency
            Some(0.9),
            Some(40),
            Some(2048),
        );
        
        // Generate response
        let response = self.ollama_client.generate(model_name, &prompt, Some(options)).await?;
        
        let elapsed = start.elapsed();
        
        // Estimate tokens/sec (rough approximation)
        let tokens_per_second = self.estimate_tokens_per_second(&response, elapsed);
        
        // Calculate accuracy based on expected keywords
        let accuracy_score = self.calculate_accuracy(&response, &query.expected_keywords);
        
        // Calculate quality score
        let quality_score = self.calculate_quality_score(&response, &validation_result);
        
        Ok(ModelTestResult {
            model_name: model_name.to_string(),
            query: query.query.clone(),
            response,
            response_time_ms: elapsed.as_millis(),
            tokens_per_second: Some(tokens_per_second),
            validation_result,
            accuracy_score,
            quality_score,
            timestamp: chrono::Utc::now().to_rfc3339(),
        })
    }

    /// Calculate accuracy based on keyword presence
    fn calculate_accuracy(&self, response: &str, expected_keywords: &[String]) -> f32 {
        if expected_keywords.is_empty() {
            return 0.5; // Neutral score if no expectations
        }
        
        let response_lower = response.to_lowercase();
        let found = expected_keywords.iter()
            .filter(|kw| response_lower.contains(&kw.to_lowercase()))
            .count();
        
        found as f32 / expected_keywords.len() as f32
    }

    /// Calculate quality score based on various factors
    fn calculate_quality_score(&self, response: &str, validation: &Option<ValidationResult>) -> f32 {
        let mut score = 0.0;
        
        // Base score on response length (not too short, not too verbose)
        let len = response.len();
        let length_score = if len < 100 {
            0.3
        } else if len < 500 {
            0.7
        } else if len < 2000 {
            0.9
        } else {
            0.6 // Too verbose
        };
        score += length_score * 0.3;
        
        // Check for code blocks
        if response.contains("```") || response.contains("void ") || response.contains("int ") {
            score += 0.2;
        }
        
        // Check for proper LPC syntax indicators
        let lpc_indicators = ["inherit", "void", "int", "string", "object", "mapping", "mixed"];
        let lpc_count = lpc_indicators.iter()
            .filter(|&ind| response.contains(ind))
            .count();
        score += (lpc_count as f32 / lpc_indicators.len() as f32) * 0.2;
        
        // Use validation result if available
        if let Some(val) = validation {
            score += val.confidence_score * 0.3;
        }
        
        score.min(1.0)
    }

    /// Estimate tokens per second
    fn estimate_tokens_per_second(&self, response: &str, elapsed: Duration) -> f32 {
        let estimated_tokens = response.split_whitespace().count() as f32 * 1.3; // rough estimate
        let seconds = elapsed.as_secs_f32();
        
        if seconds > 0.0 {
            estimated_tokens / seconds
        } else {
            0.0
        }
    }

    /// Generate comparison summary
    fn generate_summary(&self, models: &[String], results: &[ModelTestResult]) -> ComparisonSummary {
        // Find best models for different criteria
        let mut best_accuracy_model = String::new();
        let mut best_accuracy_score = 0.0;
        
        let mut best_speed_model = String::new();
        let mut best_speed = f32::MAX;
        
        let mut best_quality_model = String::new();
        let mut best_quality_score = 0.0;
        
        for model in models {
            let model_results: Vec<_> = results.iter()
                .filter(|r| r.model_name == *model)
                .collect();
            
            if model_results.is_empty() {
                continue;
            }
            
            // Average accuracy
            let avg_accuracy: f32 = model_results.iter()
                .map(|r| r.accuracy_score)
                .sum::<f32>() / model_results.len() as f32;
            
            if avg_accuracy > best_accuracy_score {
                best_accuracy_score = avg_accuracy;
                best_accuracy_model = model.clone();
            }
            
            // Average speed
            let avg_speed: f32 = model_results.iter()
                .map(|r| r.response_time_ms as f32)
                .sum::<f32>() / model_results.len() as f32;
            
            if avg_speed < best_speed {
                best_speed = avg_speed;
                best_speed_model = model.clone();
            }
            
            // Average quality
            let avg_quality: f32 = model_results.iter()
                .map(|r| r.quality_score)
                .sum::<f32>() / model_results.len() as f32;
            
            if avg_quality > best_quality_score {
                best_quality_score = avg_quality;
                best_quality_model = model.clone();
            }
        }
        
        // Choose recommended model (weighted average)
        let recommended = self.choose_recommended_model(models, results);
        
        let reasoning = format!(
            "Based on testing: {} achieved highest accuracy ({:.1}%), {} was fastest (avg {}ms), {} had best quality score ({:.1}%). Recommended model balances all factors.",
            best_accuracy_model, best_accuracy_score * 100.0,
            best_speed_model, best_speed as u32,
            best_quality_model, best_quality_score * 100.0
        );
        
        ComparisonSummary {
            best_accuracy: best_accuracy_model,
            best_speed: best_speed_model,
            best_quality: best_quality_model,
            recommended_model: recommended,
            reasoning,
        }
    }

    /// Choose recommended model based on weighted scoring
    fn choose_recommended_model(&self, models: &[String], results: &[ModelTestResult]) -> String {
        let mut best_model = String::new();
        let mut best_combined_score = 0.0;
        
        for model in models {
            let model_results: Vec<_> = results.iter()
                .filter(|r| r.model_name == *model)
                .collect();
            
            if model_results.is_empty() {
                continue;
            }
            
            let avg_accuracy: f32 = model_results.iter()
                .map(|r| r.accuracy_score)
                .sum::<f32>() / model_results.len() as f32;
            
            let avg_quality: f32 = model_results.iter()
                .map(|r| r.quality_score)
                .sum::<f32>() / model_results.len() as f32;
            
            // Normalize speed (lower is better, convert to 0-1 where 1 is best)
            let avg_time: f32 = model_results.iter()
                .map(|r| r.response_time_ms as f32)
                .sum::<f32>() / model_results.len() as f32;
            let speed_score = 1.0 / (1.0 + avg_time / 1000.0); // Normalize
            
            // Weighted combination: accuracy 40%, quality 40%, speed 20%
            let combined = avg_accuracy * 0.4 + avg_quality * 0.4 + speed_score * 0.2;
            
            if combined > best_combined_score {
                best_combined_score = combined;
                best_model = model.clone();
            }
        }
        
        best_model
    }

    /// Get default test queries for LPC development
    pub fn get_default_test_queries() -> Vec<TestQuery> {
        vec![
            TestQuery {
                query: "How do I implement a combat system in LPC?".to_string(),
                category: QueryCategory::Combat,
                expected_keywords: vec![
                    "inherit".to_string(),
                    "attack".to_string(),
                    "damage".to_string(),
                    "living".to_string(),
                ],
            },
            TestQuery {
                query: "Show me the correct syntax for query_* functions".to_string(),
                category: QueryCategory::Efun,
                expected_keywords: vec![
                    "query".to_string(),
                    "int".to_string(),
                    "return".to_string(),
                ],
            },
            TestQuery {
                query: "What's the difference between call_other() and call_out()?".to_string(),
                category: QueryCategory::Efun,
                expected_keywords: vec![
                    "call_other".to_string(),
                    "call_out".to_string(),
                    "delay".to_string(),
                    "object".to_string(),
                ],
            },
            TestQuery {
                query: "Generate a basic room inherit for the std/room.c".to_string(),
                category: QueryCategory::GeneralLPC,
                expected_keywords: vec![
                    "inherit".to_string(),
                    "room".to_string(),
                    "create".to_string(),
                    "void".to_string(),
                ],
            },
            TestQuery {
                query: "How does the LPC object inheritance system work?".to_string(),
                category: QueryCategory::ObjectSystem,
                expected_keywords: vec![
                    "inherit".to_string(),
                    "object".to_string(),
                    "virtual".to_string(),
                    "scope".to_string(),
                ],
            },
        ]
    }

    /// Save comparison results to JSON file
    pub fn save_results(&self, comparison: &ModelComparison, filename: &str) -> Result<()> {
        let json = serde_json::to_string_pretty(comparison)
            .context("Failed to serialize comparison results")?;
        
        std::fs::write(filename, json)
            .context(format!("Failed to write results to {}", filename))?;
        
        println!("Results saved to {}", filename);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_calculate_accuracy_empty() {
        let tester = create_test_tester();
        let score = tester.calculate_accuracy("test response", &[]);
        assert_eq!(score, 0.5);
    }

    #[test]
    fn test_calculate_accuracy_full_match() {
        let tester = create_test_tester();
        let response = "This response contains inherit and attack keywords";
        let keywords = vec!["inherit".to_string(), "attack".to_string()];
        let score = tester.calculate_accuracy(response, &keywords);
        assert_eq!(score, 1.0);
    }

    #[test]
    fn test_calculate_accuracy_partial() {
        let tester = create_test_tester();
        let response = "This response contains inherit only";
        let keywords = vec!["inherit".to_string(), "attack".to_string()];
        let score = tester.calculate_accuracy(response, &keywords);
        assert_eq!(score, 0.5);
    }

    #[test]
    fn test_estimate_tokens_per_second() {
        let tester = create_test_tester();
        let response = "This is a test response with several words";
        let duration = Duration::from_secs(1);
        let tps = tester.estimate_tokens_per_second(response, duration);
        assert!(tps > 0.0);
    }

    #[test]
    fn test_default_queries_count() {
        let queries = ModelTester::get_default_test_queries();
        assert_eq!(queries.len(), 5);
    }

    fn create_test_tester() -> ModelTester {
        use std::path::PathBuf;
        
        let client = OllamaClient::new().unwrap();
        let builder = PromptBuilder::new_empty(PathBuf::from("/tmp"));
        
        ModelTester::new(client, builder, None)
    }
}
