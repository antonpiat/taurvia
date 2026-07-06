import { BrowserRouter } from "react-router-dom";
import { WalletProvider } from "@/context/WalletContext";
import { AppRouter } from "@/router";

function App() {
  return (
    <WalletProvider>
      <BrowserRouter>
        <AppRouter />
      </BrowserRouter>
    </WalletProvider>
  );
}

export default App;
