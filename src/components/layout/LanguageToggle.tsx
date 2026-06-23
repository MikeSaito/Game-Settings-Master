import { useAppSettings } from "@/hooks/app/useAppSettings";
import { LanguageSelect } from "@/components/settings/LanguageSelect";

export function LanguageToggle() {
  const { settings, setLanguage } = useAppSettings();

  return <LanguageSelect value={settings.language} onChange={setLanguage} />;
}
