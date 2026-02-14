import React from 'react';
import { SortMode } from '../services/api';

interface SortSelectorProps {
  value: SortMode;
  onChange: (next: SortMode) => void;
  disabled?: boolean;
}

export default function SortSelector({ value, onChange, disabled = false }: SortSelectorProps) {
  return (
    <section className="sort-selector">
      <label htmlFor="sort-mode">Sort by</label>
      <select
        id="sort-mode"
        value={value}
        onChange={(event) => onChange(event.target.value as SortMode)}
        disabled={disabled}
      >
        <option value="Relevance">Relevance</option>
        <option value="UpdateRecencyVersionAware">Update recency (version aware)</option>
        <option value="Downloads">Downloads</option>
        <option value="CreatedDate">Created date</option>
      </select>
    </section>
  );
}
