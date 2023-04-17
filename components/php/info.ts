//new Worker(new URL("install.ts", import.meta.url), {type: "module"});

import "../helper.ts";

info({
    name: "PHP",
    available_version: () => ["7.4", "8.0"]
});

installer(async (version, target_dir) => {
    if (cp_env.target_family !== "unix" && cp_env.target_arch !== "x86_64") {
        throw new Error(`不支持的平台: ${cp_env.target_family}-${cp_env.target_arch}`);
    }
    await Deno.mkdir(target_dir, {recursive: true});
    switch (version) {
        case "7.4":
            //await fetch("https://www.php.net/distributions/php-7.4.33.tar.gz");
            console.log();
            break
        default:
            throw new Error("不支持的版本: " + version);
    }
    return true;
})
