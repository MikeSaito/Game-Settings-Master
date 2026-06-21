import { useAppSettings } from "../../hooks/useAppSettings";
import { LanguageSelect } from "../settings/LanguageSelect";

export function LanguageToggle() {
  const { settings, setLanguage } = useAppSettings();

  return <LanguageSelect value={settings.language} onChange={setLanguage} />;
}
