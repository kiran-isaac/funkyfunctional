import React, { createContext, ReactNode, useContext, useState } from "react";

interface SettingsContextType {
    isLightTheme: boolean;
    toggleTheme: () => void;

    typecheckerEnabled: boolean;
    setTypecheckerEnabled: (enabled: boolean) => void;

    preludeEnable: boolean;
    setPreludeEnable: (enabled: boolean) => void;
}

// eslint-disable-next-line react-refresh/only-export-components
export const SettingsContext = createContext<SettingsContextType | undefined>(undefined);

// Create a provider component
export const SettingsProvider: React.FC<{ children: ReactNode }> = ({ children }) => {
    const useLightMode = localStorage.getItem("darkMode") == "false" || !window.matchMedia('(prefers-color-scheme: dark)').matches;

    const [isLightTheme, setIsLightTheme] = useState(!useLightMode);
    const [typecheckerEnabled, setTypecheckerEnabled] = useState(true);
    const [preludeEnable, setPreludeEnable] = useState(true);

    window.matchMedia('(prefers-color-scheme: dark)').addEventListener('change', (event) => {
        localStorage.setItem("darkMode", event.matches ? "true" : "false");
        setIsLightTheme(event.matches);
    })

    const toggleTheme = () => {
        setIsLightTheme((prev) => {
            localStorage.setItem("darkMode", !prev ? "true" : "false");
            return !prev
        });
    };

    return (
        <SettingsContext.Provider value={{ isLightTheme, toggleTheme, typecheckerEnabled, setTypecheckerEnabled, preludeEnable, setPreludeEnable }}>
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