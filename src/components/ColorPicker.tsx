interface ColorPickerProps {
  value: string;
  onChange: (color: string) => void;
  label?: string;
}

const tailwindColors = [
  { name: 'Azul', value: '#3b82f6' },
  { name: 'Índigo', value: '#6366f1' },
  { name: 'Roxo', value: '#8b5cf6' },
  { name: 'Rosa', value: '#ec4899' },
  { name: 'Vermelho', value: '#ef4444' },
  { name: 'Laranja', value: '#f97316' },
  { name: 'Âmbar', value: '#f59e0b' },
  { name: 'Amarelo', value: '#eab308' },
  { name: 'Lima', value: '#84cc16' },
  { name: 'Verde', value: '#22c55e' },
  { name: 'Esmeralda', value: '#10b981' },
  { name: 'Turquesa', value: '#14b8a6' },
  { name: 'Ciano', value: '#06b6d4' },
  { name: 'Celeste', value: '#0ea5e9' },
  { name: 'Cinza', value: '#6b7280' },
  { name: 'Slate', value: '#64748b' },
];

export default function ColorPicker({ value, onChange, label }: ColorPickerProps) {
  return (
    <fieldset className="fieldset">
      {label && <legend className="fieldset-legend">{label}</legend>}
      <div className="grid grid-cols-8 gap-2">
        {tailwindColors.map((color) => (
          <button
            key={color.value}
            type="button"
            className={`w-10 h-10 rounded-lg transition-all hover:scale-110 ${
              value === color.value 
                ? 'ring-2 ring-offset-2 ring-primary scale-110' 
                : 'hover:ring-2 hover:ring-offset-2 hover:ring-base-content/20'
            }`}
            style={{ backgroundColor: color.value }}
            onClick={() => onChange(color.value)}
            title={color.name}
          />
        ))}
      </div>
    </fieldset>
  );
}
