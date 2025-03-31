import React, { createContext, ReactNode, useContext, useState } from "react";

interface SettingsContextType {
    isLightTheme: boolean;
    setIsLightTheme: (enabled: boolean) => void;

    typecheckerEnabled: boolean;
    setTypecheckerEnabled: (enabled: boolean) => void;

    preludeEnable: boolean;
    setPreludeEnable: (enabled: boolean) => void;
}

// Create the settings json type
type SettingsJson = {
    isLightTheme: unknown;
    typecheckerEnabled: unknown;
    preludeEnabled: unknown;
}

function useStateAndUpdateLocalStorage<T>(key: string, initialValue: T): [T, (value: T) => void] {
    const [value, setValue] = useState<T>(initialValue);
    const setStoredValue = (newValue: T) => {
        const settings = localStorage.getItem("settings");
        if (settings) {
            const settingsJson: SettingsJson = JSON.parse(settings);
            if (settings) {
                setValue(newValue);
                // @ts-expect-error fuck you typescript
                settingsJson[key] = newValue;
                localStorage.setItem("settings", JSON.stringify(settingsJson));
            }
        }
    }
    return [value, setStoredValue];
}

// eslint-disable-next-line react-refresh/only-export-components
export const SettingsContext = createContext<SettingsContextType | undefined>(undefined);

// Create a provider component
export const SettingsProvider: React.FC<{ children: ReactNode }> = ({ children }) => {
    // load the settings from local storage
    const settingsJson = localStorage.getItem("settings");
    const loadedSettings = settingsJson ? JSON.parse(settingsJson) : {
        isLightTheme: !window.matchMedia('(prefers-color-scheme: dark)').matches,
        typecheckerEnabled: true,
        preludeEnabled: true,
    };

    if (!settingsJson) {
        localStorage.setItem("settings", JSON.stringify(loadedSettings));
    }

    const [isLightTheme, setIsLightTheme] = useStateAndUpdateLocalStorage<boolean>("isLightTheme", loadedSettings.isLightTheme);
    const [typecheckerEnabled, setTypecheckerEnabled] = useStateAndUpdateLocalStorage<boolean>("typecheckerEnabled", loadedSettings.typecheckerEnabled);
    const [preludeEnable, setPreludeEnable] = useStateAndUpdateLocalStorage<boolean>("preludeEnabled", loadedSettings.preludeEnabled);

    return (
        <SettingsContext.Provider value={{
            isLightTheme,
            setIsLightTheme,
            typecheckerEnabled,
            setTypecheckerEnabled,
            preludeEnable,
            setPreludeEnable
        }}>
            {children}
        </SettingsContext.Provider>
    );
};


// eslint-disable-next-line react-refresh/only-export-components
export const useSettings = () => {
    const context = useContext(SettingsContext);
    if (!context) {
        throw new Error('useSettings must be used within a SettingsProvider');
    }
    return context;
};