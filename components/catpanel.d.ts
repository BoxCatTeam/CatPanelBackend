/// <reference types="./lib.deno.d.ts" />

declare const cp_env: {
    cp_version: string;
    cp_git_hash: string;
    target_os: string;
    target_arch: string;
    target_family: string;
    target_env: string;
    profile: string;
};

declare class Info {
    name: string | (() => string | Promise<string>);
    available_version: string[] | (() => string[] | Promise<string[]>);
}

declare function info(_: Info): void;

declare function installer(_: (version: string, target_dir: string) => Promise<boolean>): void;