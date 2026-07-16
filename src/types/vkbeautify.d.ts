declare module "vkbeautify" {
  const vkbeautify: {
    xml(text: string, indent?: string | number): string;
    json(text: string, indent?: string | number): string;
    css(text: string, indent?: string | number): string;
    sql(text: string, indent?: string | number): string;
    xmlmin(text: string, preserveComments?: boolean): string;
    jsonmin(text: string): string;
    cssmin(text: string, preserveComments?: boolean): string;
    sqlmin(text: string): string;
  };
  export default vkbeautify;
}
