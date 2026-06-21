import { useTranslation } from "react-i18next";
import type { SettingsLanguage } from "../../lib/appSettings";
import { SUPPORTED_LANGUAGES } from "../../i18n";
import { Select } from "../ds/Field";

interface Props {
  value: SettingsLanguage;
  onChange: (lang: SettingsLanguage) => void;
  className?: string;
}

export function LanguageSelect({ value, onChange, className }: Props) {
  const { t } = useTranslation("settings");

  return (
    <Select
      value={value}
      onChange={(event) => onChange(event.target.value as SettingsLanguage)}
      aria-label={t("language.aria")}
      className={className}
    >
      {SUPPORTED_LANGUAGES.map((lang) => (
        <option key={lang} value={lang}>
          {t(`language.${lang}`)}
        </option>
      ))}
    </Select>
  );
}
