import React, { useState } from 'react';

export interface SearchFormInput {
  keywords: string;
  version?: string;
  providers: string[];
  forceRefresh: boolean;
}

interface SearchFormProps {
  onSearch: (input: SearchFormInput) => void;
  isLoading: boolean;
}

export default function SearchForm({ onSearch, isLoading }: SearchFormProps) {
  const [keywords, setKeywords] = useState('');
  const [version, setVersion] = useState('');
  const [providers, setProviders] = useState<Record<string, boolean>>({
    modrinth: true,
    curseforge: true,
  });

  const selectedProviders = Object.entries(providers)
    .filter(([, enabled]) => enabled)
    .map(([provider]) => provider);

  const canSubmit = keywords.trim().length > 0 && selectedProviders.length > 0 && !isLoading;

  const submit = (forceRefresh: boolean) => {
    if (!canSubmit) {
      return;
    }

    onSearch({
      keywords,
      version: version.trim() || undefined,
      providers: selectedProviders,
      forceRefresh,
    });
  };

  return (
    <div className="search-form">
      <label htmlFor="keywords">Keywords</label>
      <input
        id="keywords"
        type="text"
        value={keywords}
        onChange={(event) => setKeywords(event.target.value)}
        placeholder="e.g. fabric performance"
      />

      <label htmlFor="version">Minecraft Version</label>
      <input
        id="version"
        type="text"
        value={version}
        onChange={(event) => setVersion(event.target.value)}
        placeholder="e.g. 1.20.1"
      />

      <fieldset>
        <legend>Providers</legend>
        <label>
          <input
            type="checkbox"
            checked={providers.modrinth}
            onChange={(event) => setProviders((current) => ({ ...current, modrinth: event.target.checked }))}
          />
          Modrinth
        </label>
        <label>
          <input
            type="checkbox"
            checked={providers.curseforge}
            onChange={(event) => setProviders((current) => ({ ...current, curseforge: event.target.checked }))}
          />
          CurseForge
        </label>
      </fieldset>

      <div className="search-form__actions">
        <button type="button" onClick={() => submit(false)} disabled={!canSubmit}>
          {isLoading ? 'Searching...' : 'Search'}
        </button>
        <button type="button" onClick={() => submit(true)} disabled={!canSubmit}>
          Refresh Results
        </button>
      </div>
    </div>
  );
}
