import { createClient } from "@connectrpc/connect";
import { AuthService as AuthServiceClient } from "@taptime/proto/taptime/services/auth_connect.js";
import type { User } from "@taptime/proto/taptime/user_pb.js";
import {
  AuthResponse,
  DeleteAccountRequest,
  LoginRequest,
  RegisterUserRequest,
  UpdateProfileRequest,
  UpdateRfidUidRequest,
  UpdateSettingsRequest,
} from "@taptime/proto/taptime/services/auth_pb.js";
import type { Uid } from "@taptime/proto/taptime/uid_pb.js";
import { transport } from "$lib/grpc";

export class AuthService {
  private static client = createClient(AuthServiceClient, transport);

  static async getUserWithToken(jwt: string): Promise<User> {
    return this.client.getUser(
      {},
      { headers: { authorization: `Bearer ${jwt}` } },
    );
  }

  static async getUser(): Promise<User> {
    const jwt = this.getStoredJwt();
    if (!jwt) throw new Error("no jwt");
    return this.getUserWithToken(jwt);
  }

  static async login(email: string, password: string): Promise<User> {
    const response = await this.client.login(
      new LoginRequest({
        email: email.trim(),
        password: password,
      }),
    );
    return this.processResponse(response);
  }

  static logout(): void {
    this.clearStoredJwt();
  }

  static async registerUser(user: User, password: string): Promise<User> {
    const response = await this.client.registerUser(
      new RegisterUserRequest({
        user: user,
        password: password,
      }),
    );
    return this.processResponse(response);
  }

  static async updateProfile(
    name: string,
    email: string,
    organization?: string,
  ): Promise<User> {
    return this.client.updateProfile(
      new UpdateProfileRequest({
        name: name.trim(),
        email: email.trim(),
        organization: organization?.trim() || undefined,
      }),
      { headers: this.authHeaders() },
    );
  }

  static async updateSettings(request: UpdateSettingsRequest): Promise<User> {
    return this.client.updateSettings(request, {
      headers: this.authHeaders(),
    });
  }

  static async updateRfidUid(rfidUid?: Uid): Promise<User> {
    return this.client.updateRfidUid(
      new UpdateRfidUidRequest({ rfidUid }),
      { headers: this.authHeaders() },
    );
  }

  static async deleteTimeData(): Promise<void> {
    await this.client.deleteTimeData({}, { headers: this.authHeaders() });
  }

  static async deleteAccount(password: string): Promise<void> {
    await this.client.deleteAccount(
      new DeleteAccountRequest({ password }),
      { headers: this.authHeaders() },
    );
    this.clearStoredJwt();
  }

  static authHeaders(): Record<string, string> {
    const jwt = this.getStoredJwt();
    if (!jwt) throw new Error("Not authenticated");
    return { authorization: `Bearer ${jwt}` };
  }

  private static processResponse(response: AuthResponse): User {
    if (!response.jwt || response.jwt.length === 0)
      throw new Error("Response has null JWT");
    if (!response.user)
      throw new Error("Response does not contain a valid user");
    this.storeJwt(response.jwt);
    return response.user;
  }

  private static getStoredJwt(): string | null {
    if (typeof localStorage === "undefined") return null;
    return localStorage.getItem("jwt");
  }

  private static clearStoredJwt(): void {
    if (typeof localStorage !== "undefined") localStorage.removeItem("jwt");
  }

  private static storeJwt(jwt: string): void {
    if (typeof localStorage !== "undefined") localStorage.setItem("jwt", jwt);
  }
}
