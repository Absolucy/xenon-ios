#import <Preferences/PSEditableTableCell.h>
#import <Preferences/PSListController.h>
#import <Preferences/PSSpecifier.h>

@interface PSEditableListController : PSListController
{

	BOOL _editable;
	BOOL _editingDisabled;
}
- (void)editDoneTapped;
- (id)_editButtonBarItem;
- (void)_setEditable:(BOOL)arg1 animated:(BOOL)arg2;
- (BOOL)performDeletionActionForSpecifier:(PSSpecifier*)specifier;
- (void)setEditingButtonHidden:(BOOL)arg1 animated:(BOOL)arg2;
- (void)setEditButtonEnabled:(BOOL)arg1;
- (void)didLock;
- (void)showController:(id)arg1 animate:(BOOL)arg2;
- (void)_updateNavigationBar;
- (id)init;
- (void)viewWillAppear:(BOOL)arg1;
- (id)tableView:(id)arg1 willSelectRowAtIndexPath:(id)arg2;
- (UITableViewCellEditingStyle)tableView:(UITableView*)tableView editingStyleForRowAtIndexPath:(NSIndexPath*)indexPath;
- (void)tableView:(id)arg1 commitEditingStyle:(long long)arg2 forRowAtIndexPath:(id)arg3;
- (void)setEditable:(BOOL)arg1;
- (void)suspend;
- (BOOL)editable;
@end

@interface PSSpecifier (Xenon)
- (SEL)action;
- (void)setName:(NSString*)arg1;
- (id)propertyForKey:(NSString*)arg1;
@end

@interface PSEditableTableCell (Xenon)
- (UILabel*)titleTextLabel;
@end

@interface OBButtonTray : UIView
@property(nonatomic, retain) UIVisualEffectView* effectView;
- (void)addButton:(id)arg1;
- (void)addCaptionText:(id)arg1;
;
@end

@interface OBBoldTrayButton : UIButton
- (void)setTitle:(id)arg1 forState:(unsigned long long)arg2;
+ (id)buttonWithType:(long long)arg1;
@end

@interface OBWelcomeController : UIViewController
@property(nonatomic, retain) UIView* viewIfLoaded;
@property(nonatomic, strong) UIColor* backgroundColor;
- (OBButtonTray*)buttonTray;
- (id)initWithTitle:(id)arg1 detailText:(id)arg2 icon:(id)arg3;
- (void)addBulletedListItemWithTitle:(id)arg1 description:(id)arg2 image:(id)arg3;
@end
