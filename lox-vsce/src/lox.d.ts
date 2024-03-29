export interface FileLocation {
    line_number: number;
    line_offset: number;
}

export interface Range {
    start: FileLocation;
    end: FileLocation;
}

export interface Location {
    path: string;
    range: Range;
}

export interface Completion {
    name: string;
    completion_type: number;
}
