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
    const [isLightTheme, setIsLightTheme] = useState(true);
    const [typecheckerEnabled, setTypecheckerEnabled] = useState(true);
    const [preludeEnable, setPreludeEnable] = useState(true);

    const toggleTheme = () => {
        setIsLightTheme((prev) => !prev);
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