import React from 'react';
import { ProviderErrorMetadata } from '../services/api';

interface DegradedModeNoticeProps {
  providerErrors: ProviderErrorMetadata[];
}

export default function DegradedModeNotice({ providerErrors }: DegradedModeNoticeProps) {
  if (providerErrors.length === 0) {
    return null;
  }

  return (
    <aside className="degraded-notice">
      <strong>Degraded mode:</strong> some providers failed.
      <ul>
        {providerErrors.map((providerError) => (
          <li key={`${providerError.provider}-${providerError.timestamp}`}>
            <strong>{providerError.provider}</strong> [{providerError.severity}] - {providerError.error}
          </li>
        ))}
      </ul>
    </aside>
  );
}
