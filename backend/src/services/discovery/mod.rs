pub mod complement_detector;
pub mod dependency_detector;
pub mod evidence_builder;
pub mod incompatibility_detector;
pub mod replacement_detector;

use crate::models::ConsolidatedModProfile;
use crate::services::cache::discovery_cache::DiscoveryCache;
use crate::services::discovery::complement_detector::ComplementDetector;
use crate::services::discovery::dependency_detector::DependencyDetector;
use crate::services::discovery::evidence_builder::{
    DetectedRelationship, DiscoveryEvidenceBuilder,
};
use crate::services::discovery::incompatibility_detector::IncompatibilityDetector;
use crate::services::discovery::replacement_detector::ReplacementDetector;
use crate::services::progress::ProgressTrackingService;
use std::collections::HashSet;
use std::sync::Arc;
use uuid::Uuid;

#[derive(Clone, Default)]
pub struct DiscoveryService {
    cache: Option<Arc<DiscoveryCache>>,
}

impl DiscoveryService {
    pub fn new(cache: Option<Arc<DiscoveryCache>>) -> Self {
        Self { cache }
    }

    pub async fn discover_for_profiles(
        &self,
        search_id: Uuid,
        profiles: &mut [ConsolidatedModProfile],
        progress_service: Arc<ProgressTrackingService>,
    ) -> usize {
        let total = profiles.len() as u32;
        progress_service
            .set_processing(search_id, "deep_discovery", 0, total, 0)
            .await;

        let mut total_relationships = 0usize;
        for (index, profile) in profiles.iter_mut().enumerate() {
            if let Some(cache) = &self.cache {
                if let Some(cached) = cache.get(&profile.id).await {
                    tracing::info!(
                        mod_profile_id = %profile.id,
                        relationships = cached.len(),
                        "Discovery cache hit"
                    );
                    total_relationships += cached.len();
                    profile.discovery_evidence = cached;
                    progress_service
                        .set_processing(
                            search_id,
                            "deep_discovery",
                            (index + 1) as u32,
                            total,
                            total_relationships as u32,
                        )
                        .await;
                    continue;
                }
                tracing::info!(mod_profile_id = %profile.id, "Discovery cache miss");
            }

            let mut relationships = Vec::new();
            relationships.extend(DependencyDetector::detect(profile));
            relationships.extend(IncompatibilityDetector::detect(profile));
            relationships.extend(ReplacementDetector::detect(profile));
            relationships.extend(ComplementDetector::detect(profile));

            let relationships = deduplicate_relationships(relationships);
            let evidence = DiscoveryEvidenceBuilder::build_all(profile.id, relationships);
            total_relationships += evidence.len();
            profile.discovery_evidence = evidence.clone();

            if let Some(cache) = &self.cache {
                let _ = cache.set(&profile.id, &evidence).await;
            }

            progress_service
                .set_processing(
                    search_id,
                    "deep_discovery",
                    (index + 1) as u32,
                    total,
                    total_relationships as u32,
                )
                .await;
        }

        progress_service
            .set_complete(search_id, total, total, total_relationships as u32)
            .await;
        tracing::info!(
            search_id = %search_id,
            profiles_processed = total,
            relationships_discovered = total_relationships,
            "Deep discovery phase completed"
        );
        total_relationships
    }
}

fn deduplicate_relationships(
    relationships: Vec<DetectedRelationship>,
) -> Vec<DetectedRelationship> {
    let mut seen = HashSet::new();
    let mut deduped = Vec::new();
    for relationship in relationships {
        let key = format!(
            "{:?}:{}",
            relationship.relationship_type,
            relationship.related_mod_name.to_lowercase()
        );
        if seen.insert(key) {
            deduped.push(relationship);
        }
    }
    deduped
}
