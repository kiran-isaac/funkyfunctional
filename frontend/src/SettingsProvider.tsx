import React, { createContext, ReactNode, useState } from "react";

interface SettingsContextType {
    isLightTheme: boolean;
    toggleTheme: () => void;
}

// Create the context with a default value
// eslint-disable-next-line react-refresh/only-export-components
export const SettingsContext = createContext<SettingsContextType | undefined>(undefined);

// Create a provider component
export const SettingsProvider: React.FC<{ children: ReactNode }> = ({ children }) => {
    const [isLightTheme, setIsLightTheme] = useState(true);

    const toggleTheme = () => {
        setIsLightTheme((prev) => !prev);
    };

    return (
        <SettingsContext.Provider value={{ isLightTheme, toggleTheme }}>
            {children}
        </SettingsContext.Provider>
    );
};