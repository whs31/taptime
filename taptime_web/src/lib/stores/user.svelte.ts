import type { User } from "@taptime/proto/taptime/user_pb.js";
import { AuthService } from "$lib/services";

let _user = $state<User | null>(null);
let _loading = $state(false);
let _error = $state<string | null>(null);
let _banNotice = $state<string | null>(null);

async function fetchUser(): Promise<void> {
  _loading = true;
  _error = null;
  try {
    _user = await AuthService.getUser();
    _banNotice = null;
  } catch (e) {
    const banNotice = AuthService.banNotice(e);
    if (banNotice) {
      _banNotice = banNotice;
      _error = null;
      _user = null;
      return;
    }
    _error = e instanceof Error ? e.message : String(e);
    _user = null;
    _banNotice = null;
  } finally {
    _loading = false;
  }
}

function clearUser(): void {
  _user = null;
  _error = null;
  _banNotice = null;
  AuthService.logout();
}

function setUser(user: User | null): void {
  _user = user;
  _error = null;
  _banNotice = null;
}

export const userStore = {
  get user(): User | null {
    return _user;
  },
  get loading(): boolean {
    return _loading;
  },
  get error(): string | null {
    return _error;
  },
  get banNotice(): string | null {
    return _banNotice;
  },
  fetch: fetchUser,
  set: setUser,
  clear: clearUser,
};
