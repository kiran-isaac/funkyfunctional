import "./input.css";

interface RunButtonProps {
    onClick: () => void;
}

function RunButton({ onClick }: RunButtonProps) {
    return <button id="RunButton" onClick={onClick}>Run</button>;
}

interface InputProps {
    onRun: () => void;
}

function Input({ onRun }: InputProps) {
    const fac = "fac :: Int -> Int\nfac n = if n <= 1 then 1 else n * (fac (n - 1))\nmain = fac 15";
    // const pair = "second (x, y) = y\nfirst (x, y) = x\npair x y = (x, y)\n\nfac:: Int -> (Int, Int)\nfac n = pair 5 (if n <= 1 then 1 else n * (second (fac (n - 1))))\nmain = second (fac 5)";
    
    return (
        <>
            <textarea id="ProgramInput" defaultValue={fac} />
            <RunButton onClick={onRun} />
        </>
    );
}

export default Input