type FiltersPanelProps = {
  filterTo: string
  filterFrom: string
  filterDataKey: string
  nodeOptions: string[]
  dataKeys: string[]
  onFilterToChange: (value: string) => void
  onFilterFromChange: (value: string) => void
  onFilterDataKeyChange: (value: string) => void
  onClear: () => void
}

const FiltersPanel = ({
  filterTo,
  filterFrom,
  filterDataKey,
  nodeOptions,
  dataKeys,
  onFilterToChange,
  onFilterFromChange,
  onFilterDataKeyChange,
  onClear,
}: FiltersPanelProps) => (
  <div className="card">
    <p>Filters</p>
    <div className="filters-row">
      <select value={filterTo} onChange={(event) => onFilterToChange(event.target.value)}>
        <option value="">All to</option>
        {nodeOptions.map((node) => (
          <option key={`to-${node}`} value={node}>
            {node}
          </option>
        ))}
      </select>
      <select value={filterFrom} onChange={(event) => onFilterFromChange(event.target.value)}>
        <option value="">All from</option>
        {nodeOptions.map((node) => (
          <option key={`from-${node}`} value={node}>
            {node}
          </option>
        ))}
      </select>
      <select value={filterDataKey} onChange={(event) => onFilterDataKeyChange(event.target.value)}>
        <option value="">All data types</option>
        {dataKeys.map((key) => (
          <option key={key} value={key}>
            {key}
          </option>
        ))}
      </select>
      <button onClick={onClear}>Clear</button>
    </div>
  </div>
)

export default FiltersPanel
