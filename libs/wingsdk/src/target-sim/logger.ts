import { Construct } from "constructs";
import { ISimulatorResource } from "./resource";
import { BaseResourceSchema } from "./schema";
import { LoggerSchema, LOGGER_TYPE } from "./schema-resources";
import { bindSimulatorResource, makeSimulatorJsClient } from "./util";
import * as cloud from "../cloud";
import { LoggerInflightMethods } from "../cloud";
import * as core from "../core";

/**
 * Simulator implementation of `cloud.Logger`.
 *
 * @inflight `@winglang/sdk.cloud.ILoggerClient`
 */
export class Logger extends cloud.Logger implements ISimulatorResource {
  constructor(scope: Construct, id: string) {
    super(scope, id);
  }

  public toSimulator(): BaseResourceSchema {
    const schema: LoggerSchema = {
      type: LOGGER_TYPE,
      path: this.node.path,
      props: {},
      attrs: {} as any,
    };
    return schema;
  }

  /** @internal */
  public _bind(host: core.IInflightHost, ops: string[]): void {
    bindSimulatorResource(__filename, this, host);
    super._bind(host, ops);
  }

  /** @internal */
  public _toInflight(): core.Code {
    return makeSimulatorJsClient(__filename, this);
  }
}

Logger._annotateInflight(LoggerInflightMethods.LOG, {});
