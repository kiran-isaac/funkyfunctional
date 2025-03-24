import React, { useContext } from "react";
import { SettingsContext } from "./SettingsProvider";

const Settings: React.FC = () => {
    const settings = useContext(SettingsContext);

    if (!settings) {
        throw new Error("Settings must be used within a SettingsProvider");
    }

    const { isLightTheme, toggleTheme } = settings;

    return (
        <div>
            <button onClick={toggleTheme}>Toggle Theme</button>
        </div>
    );
};

export default Settings;