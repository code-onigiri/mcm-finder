import React from 'react';
import { useQuery } from '@tanstack/react-query';
import CompatibilitySection from '../components/ModDetail/CompatibilitySection';
import IntegratedSummary from '../components/ModDetail/IntegratedSummary';
import MaintenanceSection from '../components/ModDetail/MaintenanceSection';
import MetadataConflictsSection from '../components/ModDetail/MetadataConflictsSection';
import RelationshipsSection from '../components/ModDetail/RelationshipsSection';
import RiskFactorsSection from '../components/ModDetail/RiskFactorsSection';
import { getModDetails } from '../services/api';

interface ModDetailPageProps {
  modId: string;
}

export default function ModDetailPage({ modId }: ModDetailPageProps) {
  const detailQuery = useQuery({
    queryKey: ['mod-details', modId],
    queryFn: () => getModDetails(modId),
  });

  if (detailQuery.isPending) {
    return (
      <main className="search-page">
        <p>Loading mod details...</p>
      </main>
    );
  }

  if (detailQuery.isError || !detailQuery.data) {
    return (
      <main className="search-page">
        <p className="search-page__error">Failed to load mod details.</p>
        <a href="#/">Back to search</a>
      </main>
    );
  }

  const details = detailQuery.data;
  return (
    <main className="search-page">
      <a href="#/" className="search-page__summary">
        ← Back to search
      </a>
      <h1>{details.canonical_name}</h1>
      <p>{details.canonical_slug}</p>

      <IntegratedSummary summary={details.integrated_summary} />
      <CompatibilitySection
        compatibility={details.compatibility_assessment}
        targetVersion={details.all_supported_versions[0]}
      />
      <MaintenanceSection maintenance={details.maintenance_signals} mostRecentUpdate={details.most_recent_update} />
      <RiskFactorsSection risks={details.adoption_risks} />
      <RelationshipsSection relationships={details.discovery_evidence} />
      <MetadataConflictsSection conflicts={details.metadata_conflicts} />
    </main>
  );
}
