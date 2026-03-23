import { useState, useRef, useEffect } from 'react';
import * as FaIcons from 'react-icons/fa';
import * as MdIcons from 'react-icons/md';
import { FaChevronDown } from 'react-icons/fa';

interface IconPickerProps {
  value: string;
  onChange: (icon: string) => void;
  label?: string;
  icons?: string[];
  iconLib?: 'fa' | 'md';
}

const FA_ICON_LIBRARY = [
  'FaWallet', 'FaMoneyBillWave', 'FaUniversity', 'FaPiggyBank', 'FaCreditCard',
  'FaCoins', 'FaChartLine', 'FaDollarSign', 'FaShoppingCart', 'FaShoppingBag',
  'FaUtensils', 'FaCoffee', 'FaBeer', 'FaPizza', 'FaHamburger',
  'FaHome', 'FaCar', 'FaBus', 'FaPlane', 'FaTrain',
  'FaGasPump', 'FaBolt', 'FaTint', 'FaPhone', 'FaWifi',
  'FaTv', 'FaLaptop', 'FaMobileAlt', 'FaGamepad', 'FaMusic',
  'FaFilm', 'FaBook', 'FaGraduationCap', 'FaBriefcase', 'FaBuilding',
  'FaHeart', 'FaHeartbeat', 'FaMedkit', 'FaPills', 'FaDumbbell',
  'FaRunning', 'FaBicycle', 'FaFutbol', 'FaBasketballBall', 'FaSwimmer',
  'FaTshirt', 'FaShoePrints', 'FaGem', 'FaGift', 'FaPaw'
];

function resolveIcon(iconName: string) {
  if (iconName.startsWith('Md')) {
    return (MdIcons as unknown as Record<string, React.ElementType>)[iconName];
  }
  return (FaIcons as unknown as Record<string, React.ElementType>)[iconName];
}

export default function IconPicker({ value, onChange, label, icons, iconLib = 'fa' }: IconPickerProps) {
  const [isOpen, setIsOpen] = useState(false);
  const [searchTerm, setSearchTerm] = useState('');
  const dropdownRef = useRef<HTMLDivElement>(null);

  const ICON_LIBRARY = icons ?? (iconLib === 'md' ? [] : FA_ICON_LIBRARY);
  const SelectedIcon = resolveIcon(value) || FaIcons.FaQuestion;

  useEffect(() => {
    const handleClickOutside = (event: MouseEvent) => {
      if (dropdownRef.current && !dropdownRef.current.contains(event.target as Node)) {
        setIsOpen(false);
      }
    };
    document.addEventListener('mousedown', handleClickOutside);
    return () => document.removeEventListener('mousedown', handleClickOutside);
  }, []);

  const filteredIcons = ICON_LIBRARY.filter(iconName =>
    iconName.toLowerCase().includes(searchTerm.toLowerCase())
  );

  return (
    <fieldset className="fieldset">
      {label && <legend className="fieldset-legend">{label}</legend>}
      <div ref={dropdownRef} className="relative">
        <button
          type="button"
          className="input flex items-center justify-between w-full h-auto min-h-12 py-2 cursor-pointer hover:border-primary transition-colors"
          onClick={() => setIsOpen(!isOpen)}
        >
          <div className="flex items-center gap-3">
            <div className="w-10 h-10 rounded-lg bg-primary/10 flex items-center justify-center">
              <SelectedIcon className="text-2xl text-primary" />
            </div>
            <span className="font-medium">{value}</span>
          </div>
          <FaChevronDown className={`transition-transform ${isOpen ? 'rotate-180' : ''}`} />
        </button>

        {isOpen && (
          <div className="absolute z-50 w-full mt-2 bg-base-100 border border-base-300 rounded-lg shadow-xl">
            <div className="p-3 border-b border-base-300">
              <input
                type="text"
                className="input input-sm w-full"
                placeholder="Buscar ícone..."
                value={searchTerm}
                onChange={(e) => setSearchTerm(e.target.value)}
                onClick={(e) => e.stopPropagation()}
              />
            </div>
            <div className="p-3 max-h-80 overflow-y-auto">
              <div className="grid grid-cols-8 gap-2">
                {filteredIcons.map((iconName) => {
                  const Icon = resolveIcon(iconName) || FaIcons.FaQuestion;
                  return (
                    <button
                      key={iconName}
                      type="button"
                      className={`w-10 h-10 rounded-lg flex items-center justify-center transition-all hover:scale-110 ${
                        value === iconName
                          ? 'bg-primary text-primary-content ring-2 ring-primary scale-110'
                          : 'bg-base-200 hover:bg-base-300'
                      }`}
                      onClick={() => {
                        onChange(iconName);
                        setIsOpen(false);
                        setSearchTerm('');
                      }}
                      title={iconName}
                    >
                      <Icon className="text-xl" />
                    </button>
                  );
                })}
              </div>
              {filteredIcons.length === 0 && (
                <div className="text-center py-8 text-base-content/50">
                  Nenhum ícone encontrado
                </div>
              )}
            </div>
          </div>
        )}
      </div>
    </fieldset>
  );
}
