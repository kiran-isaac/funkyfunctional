import logo from './logo.svg';
import './App.css';
import * as wasm from "hello-wasm";

function App() {
  return (
    <div className="App">
      <header className="App-header">
        <img src={logo} className="App-logo" alt="logo" />
        <p>
          {wasm.hello_world()}
        </p>
      </header>
    </div>
  );
}

export default App;