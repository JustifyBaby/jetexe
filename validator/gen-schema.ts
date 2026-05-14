/// <reference types="bun" />

import { z } from "zod"; // 💡 もしschema.ts側でインポートしている場合はここでもzを合わせます
import { TestSuiteSchema } from "./schema";

const schema = z.toJSONSchema(TestSuiteSchema);

await Bun.write("./validator/schema.json", JSON.stringify(schema, null, 2));

console.log("schema.json を正常に生成しました！");
