/**
 * Copyright (c) 2024 DSR Corporation, Denver, Colorado.
 * https://www.dsr-corporation.com
 * SPDX-License-Identifier: Apache-2.0
 */

import fs from 'fs'

export function writeJson(data: Record<string, unknown>, outFile: string) {
  const content = JSON.stringify(data, null, '\t')
  fs.writeFileSync(outFile, content)
}
