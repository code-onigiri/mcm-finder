import React from 'react';
import { DiscoveryEvidenceItem } from '../../services/api';

interface RelationshipsSectionProps {
  relationships: DiscoveryEvidenceItem[];
}

export default function RelationshipsSection({ relationships }: RelationshipsSectionProps) {
  return (
    <section className="mod-detail-section">
      <h3>Relationships</h3>
      {relationships.length === 0 ? (
        <p>No relationships discovered.</p>
      ) : (
        <ul>
          {relationships.map((relationship) => (
            <li key={relationship.id}>
              <strong>{relationship.relationship_type}</strong> → {relationship.related_mod_name} (
              {Math.round(relationship.confidence * 100)}%)
              <div>{relationship.context}</div>
            </li>
          ))}
        </ul>
      )}
    </section>
  );
}
