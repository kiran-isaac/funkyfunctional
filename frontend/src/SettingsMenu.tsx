import React, { useContext, useEffect, useRef } from "react";
import { SettingsContext } from "./SettingsProvider";
import "./settings.css";

interface SettingsProps {
    settingsIsVisible: boolean;
    dismissSettings: () => void;
}

const Settings: React.FC<SettingsProps> = ({ settingsIsVisible, dismissSettings }) => {
    const settings = useContext(SettingsContext);
    const settingsRef = useRef<HTMLDivElement>(null);

    useEffect(() => {
        const handleClickOutside = (event: MouseEvent) => {
            if (settingsRef.current && !settingsRef.current.contains(event.target as Node)) {
                dismissSettings();
            }
        };

        document.addEventListener('mousedown', handleClickOutside);
        return () => {
            document.removeEventListener('mousedown', handleClickOutside);
        };
    }, [dismissSettings]);


    if (!settingsIsVisible) {
        return <></>;
    }

    if (!settings) {
        throw new Error("Settings must be used within a SettingsProvider");
    }

    const { setIsLightTheme, isLightTheme, typecheckerEnabled, setTypecheckerEnabled, preludeEnable, setPreludeEnable } = settings;

    return (
        <div id="settings" className={settingsIsVisible ? "visible" : "hidden"} ref={settingsRef}>
            <button id="dismiss" onClick={dismissSettings}>X</button>
            <div>
                <h2>UI Settings</h2>
                <button onClick={() => setIsLightTheme(!isLightTheme)}>
                    {isLightTheme ? "Switch to Dark Mode" : "Switch to Light Mode"}
                </button>
                <br />
                <h2>Language Settings</h2>
                <button onClick={() => setTypecheckerEnabled(!typecheckerEnabled)}>
                    {typecheckerEnabled ? "Disable Type Checker" : "Enable Type Checker"}
                </button>
                <button onClick={() => setPreludeEnable(!preludeEnable)}>
                    {preludeEnable ? "Disable Prelude" : "Enable Prelude"}
                </button>
            </div>
        </div>
    );
};

export default Settings;