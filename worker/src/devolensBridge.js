// Devolens API Worker Bridge

const DEFAULT_TIMEOUT_MS = 5000;

function redactText(text, token, key) {
  if (!text) return "";
  let result = String(text);
  if (token) {
    result = result.replaceAll(token, "[redacted-token]");
  }
  if (key) {
    result = result.replaceAll(key, "[redacted-key]");
  }
  return result;
}

async function callDevolensAPI(env, endpoint, params, options = {}) {
  let token;
  let tokenVarName = "";
  if (options.tokenScope === "support") {
    tokenVarName = "DEVOLENS_SUPPORT_TOKEN";
    token = env.DEVOLENS_SUPPORT_TOKEN;
    if (!token && env.DEVOLENS_ACCESS_TOKEN) {
      token = env.DEVOLENS_ACCESS_TOKEN;
    }
  } else if (endpoint === "/api/key/CreateKey" || endpoint === "/api/key/BlockKey") {
    tokenVarName = "DEVOLENS_WEBHOOK_TOKEN";
    token = env.DEVOLENS_WEBHOOK_TOKEN;
    if (!token && env.DEVOLENS_ACCESS_TOKEN) {
      token = env.DEVOLENS_ACCESS_TOKEN;
    }
  } else if (endpoint === "/api/key/Deactivate") {
    tokenVarName = "DEVOLENS_CLIENT_TOKEN";
    token = env.DEVOLENS_CLIENT_TOKEN;
    if (!token && env.DEVOLENS_ACCESS_TOKEN) {
      token = env.DEVOLENS_ACCESS_TOKEN;
    }
  } else {
    tokenVarName = "DEVOLENS_SUPPORT_TOKEN";
    token = env.DEVOLENS_SUPPORT_TOKEN;
    if (!token && env.DEVOLENS_ACCESS_TOKEN) {
      token = env.DEVOLENS_ACCESS_TOKEN;
    }
  }

  const productId = env.DEVOLENS_PRODUCT_ID;
  const key = params.Key;

  if (!token || !productId) {
    return {
      ok: false,
      code: "devolens_error",
      message: `Devolens configuration (${tokenVarName}/DEVOLENS_PRODUCT_ID) is missing.`,
      status: 400,
      retryable: false
    };
  }

  const baseUrl = env.DEVOLENS_BASE_URL || "https://api.cryptolens.ph";
  const devolensUrl = new URL(`${baseUrl}${endpoint}`);

  const form = new URLSearchParams();
  form.append("token", token);
  form.append("ProductId", productId);
  for (const [k, v] of Object.entries(params)) {
    form.append(k, String(v));
  }

  const controller = new AbortController();
  const timeoutId = setTimeout(() => controller.abort(), DEFAULT_TIMEOUT_MS);

  try {
    const res = await fetch(devolensUrl.toString(), {
      method: "POST",
      headers: { "content-type": "application/x-www-form-urlencoded" },
      body: form.toString(),
      signal: controller.signal
    });

    clearTimeout(timeoutId);

    if (!res.ok) {
      return {
        ok: false,
        code: "worker_unreachable",
        message: `Devolens API returned HTTP error ${res.status}`,
        status: 503,
        retryable: true
      };
    }

    let data;
    try {
      data = await res.json();
    } catch (parseErr) {
      return {
        ok: false,
        code: "worker_unreachable",
        message: "Failed to parse Devolens API response JSON.",
        status: 503,
        retryable: true
      };
    }

    if (data.result !== 0) {
      return {
        ok: false,
        code: "devolens_error",
        message: data.message || `Devolens API returned error result ${data.result}`,
        status: 502,
        retryable: false
      };
    }

    return { ok: true, data };
  } catch (err) {
    clearTimeout(timeoutId);
    const rawMessage = err instanceof Error ? err.message : String(err);
    const safeMessage = redactText(rawMessage, token, key);

    // If it was aborted by our timeout controller or is a network error, classify as retryable
    const isAbort = err.name === "AbortError" || rawMessage.includes("abort");
    return {
      ok: false,
      code: "worker_unreachable",
      message: `Failed to contact Devolens: ${safeMessage}`,
      status: 503,
      retryable: isAbort || true
    };
  }
}

export async function createDevolensKey(env, licenseKey) {
  return callDevolensAPI(env, "/api/key/CreateKey", { Key: licenseKey });
}

export async function blockDevolensKey(env, licenseKey) {
  return callDevolensAPI(env, "/api/key/BlockKey", { Key: licenseKey });
}

export async function blockDevolensKeyForPrivacy(env, licenseKey) {
  return callDevolensAPI(env, "/api/key/BlockKey", { Key: licenseKey }, { tokenScope: "support" });
}

export async function deactivateDevolensKey(env, licenseKey, machineCode) {
  return callDevolensAPI(env, "/api/key/Deactivate", { Key: licenseKey, MachineCode: machineCode });
}
