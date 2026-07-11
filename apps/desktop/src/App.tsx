import { HashRouter } from "react-router-dom";
import { WalletProvider } from "@/context/WalletContext";
import { LayoutModeProvider } from "@/lib/appView";
import { AppRouter } from "@/router";

function App() {
  return (
    <HashRouter>
      <LayoutModeProvider>
        <WalletProvider>
          <AppRouter />
        </WalletProvider>
      </LayoutModeProvider>
    </HashRouter>
  );
}

export default App;
