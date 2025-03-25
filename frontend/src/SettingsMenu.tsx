import React, { useContext } from "react";
import { SettingsContext } from "./SettingsProvider";

interface SettingsProps {
    settingsIsVisible: boolean;
}

const Settings: React.FC<SettingsProps> = ({settingsIsVisible})=> {
    const settings = useContext(SettingsContext);

    if (!settingsIsVisible) {
        return <></>;
    }

    if (!settings) {
        throw new Error("Settings must be used within a SettingsProvider");
    }

    const { toggleTheme } = settings;

    return (
        <div>
            <button onClick={toggleTheme}>Toggle Theme</button>
        </div>
    );
};

export default Settings;