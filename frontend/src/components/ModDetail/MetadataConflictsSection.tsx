import React from 'react';
import { MetadataConflict } from '../../services/api';

interface MetadataConflictsSectionProps {
  conflicts: MetadataConflict[];
}

export default function MetadataConflictsSection({ conflicts }: MetadataConflictsSectionProps) {
  return (
    <section className="mod-detail-section">
      <h3>Metadata Conflicts</h3>
      {conflicts.length === 0 ? (
        <p>No metadata conflicts found across providers.</p>
      ) : (
        <ul>
          {conflicts.map((conflict) => (
            <li key={`${conflict.field}-${conflict.severity}`}>
              <strong>{conflict.field}</strong> <span>[{conflict.severity}]</span>
              <ul>
                {Object.entries(conflict.values).map(([source, value]) => (
                  <li key={`${conflict.field}-${source}`}>
                    {source}: {value}
                  </li>
                ))}
              </ul>
            </li>
          ))}
        </ul>
      )}
    </section>
  );
}
