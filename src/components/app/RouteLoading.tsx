import { useTranslation } from "react-i18next";
import { StartupScreen } from "./StartupScreen";

export function RouteLoading() {
  const { t } = useTranslation("startup");
  return <StartupScreen embedded title={t("workspaceLoading")} />;
}
