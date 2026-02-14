import React from 'react';
import { CompatibilityAssessment } from '../../services/api';

interface CompatibilitySectionProps {
  compatibility: CompatibilityAssessment;
  targetVersion?: string;
}

export default function CompatibilitySection({ compatibility, targetVersion }: CompatibilitySectionProps) {
  const details = formatCompatibility(compatibility);

  return (
    <section className="mod-detail-section">
      <h3>Compatibility</h3>
      <p>{details.label}</p>
      {targetVersion && <p>Target version: {targetVersion}</p>}
      {details.items.length > 0 && (
        <ul>
          {details.items.map((item) => (
            <li key={item}>{item}</li>
          ))}
        </ul>
      )}
    </section>
  );
}

function formatCompatibility(compatibility: CompatibilityAssessment): { label: string; items: string[] } {
  if (typeof compatibility === 'string') {
    return { label: compatibility, items: [] };
  }

  if ('PartiallyCompatible' in compatibility) {
    return {
      label: 'Partially Compatible',
      items: compatibility.PartiallyCompatible.issues || [],
    };
  }

  if ('Incompatible' in compatibility) {
    return {
      label: 'Incompatible',
      items: compatibility.Incompatible.reasons || [],
    };
  }

  return { label: 'FullyCompatible', items: [] };
}
