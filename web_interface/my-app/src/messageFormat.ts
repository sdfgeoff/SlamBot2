export enum DiagnosticStatus {
    Ok = 0,
    Warn = 1,
    Error = 2,
    Stale = 3,
}

export interface DiagnosticKeyValue {
    key: string;
    value: string;
}

export interface PacketFormat {
    to: number | null;
    from: number | null;
    time: bigint;
    id: number;
    data: {
        "OdometryDelta": {
            start_time: bigint;
            end_time: bigint;
            delta_position: [number, number];
            delta_orientation: number;
        },
    } | {
        "DiagnosticMsg": {
            level: DiagnosticStatus;
            name: string;
            message: string;
            values: DiagnosticKeyValue[];
        }
    } | {
        "SubscriptionRequest": {
            topics: string[];
        }
    } | { [key: string]: unknown};
}
