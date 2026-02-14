import React from 'react';

const VERSION_OPTIONS = ['', '1.21.1', '1.20.1', '1.19.2', '1.18.2', '1.16.5', '1.12.2', '1.7.10'];
const LOADER_OPTIONS = ['fabric', 'forge', 'quilt', 'neoforge'];
const CATEGORY_OPTIONS = ['optimization', 'utility', 'technology', 'worldgen', 'adventure', 'library'];
const RECENCY_OPTIONS = [
  { label: 'Any time', value: '' },
  { label: '30 days', value: '30' },
  { label: '90 days', value: '90' },
  { label: '180 days', value: '180' },
  { label: '365 days', value: '365' },
];

export interface FilterPanelState {
  minecraft_version?: string;
  loaders: string[];
  categories: string[];
  min_downloads?: number;
  open_source_only: boolean;
  update_recency_window_days?: number;
}

interface FilterPanelProps {
  value: FilterPanelState;
  onChange: (next: FilterPanelState) => void;
  disabled?: boolean;
}

function toggleListValue(values: string[], value: string): string[] {
  if (values.includes(value)) {
    return values.filter((entry) => entry !== value);
  }

  return [...values, value];
}

export default function FilterPanel({ value, onChange, disabled = false }: FilterPanelProps) {
  const minDownloads = value.min_downloads ?? 0;

  return (
    <section className="filter-panel">
      <h2>Advanced Filters</h2>

      <label htmlFor="filter-version">Minecraft Version</label>
      <select
        id="filter-version"
        value={value.minecraft_version ?? ''}
        onChange={(event) =>
          onChange({
            ...value,
            minecraft_version: event.target.value || undefined,
          })
        }
        disabled={disabled}
      >
        {VERSION_OPTIONS.map((version) => (
          <option key={version || 'any'} value={version}>
            {version || 'Any version'}
          </option>
        ))}
      </select>

      <fieldset>
        <legend>Loaders</legend>
        {LOADER_OPTIONS.map((loader) => (
          <label key={loader}>
            <input
              type="checkbox"
              checked={value.loaders.includes(loader)}
              onChange={() => onChange({ ...value, loaders: toggleListValue(value.loaders, loader) })}
              disabled={disabled}
            />
            {loader}
          </label>
        ))}
      </fieldset>

      <fieldset>
        <legend>Categories</legend>
        {CATEGORY_OPTIONS.map((category) => (
          <label key={category}>
            <input
              type="checkbox"
              checked={value.categories.includes(category)}
              onChange={() =>
                onChange({
                  ...value,
                  categories: toggleListValue(value.categories, category),
                })
              }
              disabled={disabled}
            />
            {category}
          </label>
        ))}
      </fieldset>

      <label htmlFor="filter-downloads">Minimum Downloads: {minDownloads.toLocaleString()}</label>
      <input
        id="filter-downloads"
        type="range"
        min={0}
        max={1000000}
        step={5000}
        value={minDownloads}
        onChange={(event) => {
          const next = Number(event.target.value);
          onChange({
            ...value,
            min_downloads: next > 0 ? next : undefined,
          });
        }}
        disabled={disabled}
      />

      <label>
        <input
          type="checkbox"
          checked={value.open_source_only}
          onChange={(event) => onChange({ ...value, open_source_only: event.target.checked })}
          disabled={disabled}
        />
        Open source only
      </label>

      <label htmlFor="filter-recency">Updated Within</label>
      <select
        id="filter-recency"
        value={value.update_recency_window_days?.toString() ?? ''}
        onChange={(event) => {
          const next = event.target.value;
          onChange({
            ...value,
            update_recency_window_days: next ? Number(next) : undefined,
          });
        }}
        disabled={disabled}
      >
        {RECENCY_OPTIONS.map((option) => (
          <option key={option.label} value={option.value}>
            {option.label}
          </option>
        ))}
      </select>
    </section>
  );
}
