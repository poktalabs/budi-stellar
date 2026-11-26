// Budi sign-in for the aula: the same Privy app as the Budi app, mounted as a
// small React island. The rest of the aula is plain TypeScript and talks to it
// through `sesion` (login, logout, access token) and the "sesion" event.
import { PrivyProvider, usePrivy } from "@privy-io/react-auth";
import { useEffect } from "react";
import { createRoot } from "react-dom/client";

export interface Sesion {
  lista: boolean;
  dentro: boolean;
  correo?: string;
  entrar(): void;
  salir(): Promise<void>;
  token(): Promise<string | null>;
}

export const sesion: Sesion = {
  lista: false,
  dentro: false,
  entrar: () => {},
  salir: async () => {},
  token: async () => null,
};

function Puente() {
  const { ready, authenticated, user, login, logout, getAccessToken } = usePrivy();
  useEffect(() => {
    Object.assign(sesion, {
      lista: ready,
      dentro: ready && authenticated,
      correo: user?.email?.address ?? user?.google?.email,
      entrar: login,
      salir: logout,
      token: getAccessToken,
    });
    window.dispatchEvent(new Event("sesion"));
  }, [ready, authenticated, user, login, logout, getAccessToken]);
  return null;
}

export function montarSesion(appId: string) {
  const raiz = document.createElement("div");
  document.body.append(raiz);
  createRoot(raiz).render(
    <PrivyProvider
      appId={appId}
      config={{
        loginMethods: ["google", "email"],
        appearance: { theme: "light", accentColor: "#c4177a", landingHeader: "Entra con tu cuenta de Budi" },
        embeddedWallets: { ethereum: { createOnLogin: "off" }, solana: { createOnLogin: "off" } },
      }}
    >
      <Puente />
    </PrivyProvider>,
  );
}
