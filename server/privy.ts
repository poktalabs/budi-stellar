// Budi sign-in: Privy access tokens are ES256 JWTs (iss `privy.io`, aud = the
// app id), checked against the app's JWKS, cached for an hour. Same scheme as
// the Budi API, so a person signs in once with their Budi account.
import { createRemoteJWKSet, jwtVerify } from "jose";

export type Verify = (token: string) => Promise<string>;

/** Returns a function that turns an access token into the person's Privy id. */
export function privyVerifier(appId: string): Verify {
  const jwks = createRemoteJWKSet(new URL(`https://api.privy.io/v1/apps/${appId}/jwks.json`), {
    cacheMaxAge: 60 * 60 * 1000,
  });
  return async (token) => {
    const { payload } = await jwtVerify(token, jwks, { algorithms: ["ES256"], issuer: "privy.io", audience: appId });
    if (!payload.sub) throw new Error("token without sub");
    return payload.sub;
  };
}
