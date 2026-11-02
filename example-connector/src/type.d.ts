export type ClientConnectorDefinition = {
    code: string,
    position: number[],
    head: number[],
    is_force_muted: boolean,
    display_name?: string,
    group?: string,
    actions?: {
        toggle_muted?: boolean,
        toggle_deafened?: boolean,
    }
}

export type ConnectionDefinition = {
    code: string,
    display_name: string,
    is_connector: boolean,
    is_force_muted: boolean,
    is_muted: boolean,
    is_deafened: boolean
}

export type SyncConnectorStateRequest = {
    clients: ClientConnectorDefinition[];
}

export type SyncConnectorStateResponse = {
    connections: ConnectionDefinition[];
}

export type AuthRequest = {
    is_connector: boolean,
    auth_key: string,
    channel: string,
}

export type AuthResponse = {
    max_distance: number,
    app_version: string
}

export type ErrorResponse = {
    message: string,
    details: string,
    code: number
}

export type MessageKind = 'auth' | 'error' | 'sync_connector_state';
export type MessageData = {
    kind: MessageKind,
    data: any,
    transaction_uuid: string
}