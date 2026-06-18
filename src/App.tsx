import { useEffect, useState } from "react";
import { useTranslation } from "react-i18next";
import { AccountSwitcher } from "./components/AccountSwitcher";
import { runUpdate, type UpdateStatus } from "./lib/updater";
import "./App.css";

function App() {
  const { t } = useTranslation();
  const [update, setUpdate] = useState<UpdateStatus>({ state: "idle" });

  useEffect(() => {
    runUpdate(setUpdate);
  }, []);

  return (
    <>
      {(update.state === "downloading" || update.state === "ready") && (
        <div className="update-banner">
          {update.state === "downloading"
            ? t("update.downloading", { version: update.version })
            : t("update.ready")}
        </div>
      )}
      <AccountSwitcher />
    </>
  );
}

export default App;
