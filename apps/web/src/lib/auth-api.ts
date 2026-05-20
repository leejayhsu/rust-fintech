export type User = {
  id: string;
  email: string;
  created_at: string;
};

export type AuthCredentials = {
  email: string;
  password: string;
};

type ApiError = {
  code: string;
  desc: string;
};

type ApiResponse<T> = {
  success: boolean;
  error?: ApiError | null;
  data: T | null;
};

const API_BASE = "/api/v1";

export async function signin(credentials: AuthCredentials) {
  return authRequest<User>("/auth/signin", credentials);
}

export async function signup(credentials: AuthCredentials) {
  return authRequest<User>("/auth/signup", credentials);
}

export async function getCurrentUser() {
  return apiRequest<User>("/auth/me");
}

async function authRequest<T>(path: string, credentials: AuthCredentials) {
  return apiRequest<T>(path, {
    method: "POST",
    body: JSON.stringify(credentials),
  });
}

async function apiRequest<T>(path: string, init?: RequestInit) {
  const response = await fetch(`${API_BASE}${path}`, {
    credentials: "include",
    headers: {
      "Content-Type": "application/json",
      ...init?.headers,
    },
    ...init,
  });

  const body = (await response.json()) as ApiResponse<T>;

  if (!response.ok || !body.success) {
    throw new Error(body.error?.desc ?? "Request failed");
  }

  if (body.data === null) {
    throw new Error("Response did not include data");
  }

  return body.data;
}
