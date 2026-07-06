import { HashRouter } from "react-router-dom";
import { WalletProvider } from "@/context/WalletContext";
import { AppRouter } from "@/router";

function App() {
  return (
    <WalletProvider>
      <HashRouter>
        <AppRouter />
      </HashRouter>
    </WalletProvider>
  );
}

export default App;
