import React from 'react';
import { RiskFactor } from '../../services/api';

interface RiskFactorsSectionProps {
  risks: RiskFactor[];
}

export default function RiskFactorsSection({ risks }: RiskFactorsSectionProps) {
  return (
    <section className="mod-detail-section">
      <h3>Risk Factors</h3>
      {risks.length === 0 ? (
        <p>No significant risk factors detected.</p>
      ) : (
        <ul>
          {risks.map((risk) => (
            <li key={`${risk.risk_type}-${risk.description}`}>
              <strong>{risk.risk_type}</strong> <span>[{risk.severity}]</span> - {risk.description}
            </li>
          ))}
        </ul>
      )}
    </section>
  );
}
