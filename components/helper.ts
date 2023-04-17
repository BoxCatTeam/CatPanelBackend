/// <reference path="./catpanel.d.ts" />

import { ensureFile } from "https://deno.land/std/fs/ensure_file.ts";

async function downloadFile(src: string, dest: string) {
    if (!(src.startsWith("http://") || src.startsWith("https://"))) {
        throw new TypeError("URL must start with be http:// or https://");
    }
    const resp = await fetch(src);
    if (!resp.ok) {
        throw new Deno.errors.BadResource(
            `Request failed with status ${resp.status}`,
        );
    } else if (!resp.body) {
        throw new Deno.errors.UnexpectedEof(
            `The download url ${src} doesn't contain a file to download`,
        );
    } else if (resp.status === 404) {
        throw new Deno.errors.NotFound(
            `The requested url "${src}" could not be found`,
        );
    }

    await ensureFile(dest);
    const file = await Deno.open(dest, { truncate: true, write: true });
    await resp.body.pipeTo(file.writable);
}