#import <Cocoa/Cocoa.h>
int main(void) {
 @autoreleasepool {
  NSApplication *app=[NSApplication sharedApplication];
  [app setActivationPolicy:NSApplicationActivationPolicyRegular];
  NSWindow *window=[[NSWindow alloc] initWithContentRect:NSMakeRect(0,0,1024,720) styleMask:NSWindowStyleMaskTitled|NSWindowStyleMaskClosable|NSWindowStyleMaskResizable backing:NSBackingStoreBuffered defer:NO];
  [window setTitle:@"Memory comparison Cocoa"];
  [window setBackgroundColor:[NSColor blackColor]];
  [window orderFront:nil];
  [app run];
 }
 return 0;
}
