import React, { useContext, useEffect, useRef } from "react";
import { SettingsContext } from "./SettingsProvider";
import "./settings.css";

interface SettingsProps {
    settingsIsVisible: boolean;
    dismissSettings: () => void;
}

const Settings: React.FC<SettingsProps> = ({settingsIsVisible, dismissSettings})=> {
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

    const { toggleTheme } = settings;    

    return (
        <div id="settings" className={settingsIsVisible ? "visible" : "hidden"} ref={settingsRef}>
            <button id="dismiss" onClick={() => {
                dismissSettings();
            }}>X</button>
            <div>
                <h2>UI Settings</h2>
                <button onClick={toggleTheme}>Toggle Theme</button>
                <br/>
                <h2></h2>
            </div>
        </div>
    );
};

export default Settings;