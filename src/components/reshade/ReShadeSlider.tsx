interface Props {
  label: string;
  value: number;
  min: number;
  max: number;
  step: number;
  disabled?: boolean;
  onChange: (value: number) => void;
}

export function ReShadeSlider({
  label,
  value,
  min,
  max,
  step,
  disabled,
  onChange,
}: Props) {
  const inputId = `reshade-slider-${label.replace(/\s+/g, "-").toLowerCase()}`;
  return (
    <div>
      <div className="mb-1.5 flex justify-between text-sm">
        <label htmlFor={inputId} className="text-[var(--color-text)]">
          {label}
        </label>
        <span className="tabular-nums text-muted">{value.toFixed(2)}</span>
      </div>
      <input
        id={inputId}
        type="range"
        min={min}
        max={max}
        step={step}
        value={value}
        disabled={disabled}
        aria-label={label}
        onChange={(e) => onChange(Number.parseFloat(e.target.value))}
        className="w-full"
      />
    </div>
  );
}
